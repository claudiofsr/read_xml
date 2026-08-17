//! # Algoritmos de Grafos e Fecho Transitivo Fiscal
//!
//! Modela as relações entre CT-es (subcontratações, redespachos, complementares)
//! e mapeia a relação bipartida CT-e <-> NF-e utilizando um algoritmo de ponto fixo.

use crate::xml_structs::{cte::InfoCte, nfe::InfoNfe};
use rayon::prelude::*;
use std::{
    cmp::Ordering,
    collections::{BTreeMap, HashMap, HashSet},
};

/// Identificador de documento fiscal para correlação indexada por validade/cancelamento.
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KeyDoc {
    pub chave: String,
    pub valido: bool,
}

impl KeyDoc {
    pub fn new<T>(chave: T, valido: bool) -> Self
    where
        T: ToString,
    {
        Self {
            chave: chave.to_string(),
            valido,
        }
    }
}

/// Estrutura consolidada de correlações cruzadas de documentos fiscais.
#[derive(Debug, Default, Clone)]
pub struct Correlacoes {
    pub cte_info: BTreeMap<KeyDoc, Vec<InfoCte>>,
    pub nfe_info: BTreeMap<KeyDoc, Vec<InfoNfe>>,
    pub cte_nfes: HashMap<String, HashSet<String>>,
    pub nfe_ctes: HashMap<String, HashSet<String>>,
}

/// # Estrutura Disjoint-Set Union (DSU / Union-Find)
///
/// Implementação de conjuntos disjuntos de alta performance utilizada para encontrar
/// **Componentes Conexas** (grupos de documentos correlacionados) em tempo quase linear O(V + E).
///
/// ---
///
/// ## A Metáfora Didática: O "Líder de Turma"
///
/// Pense em cada documento fiscal (CT-e) como uma pessoa em um pátio:
///
/// 1. **Início**: Cada pessoa forma um grupo de um integrante só e é a líder de si mesma.
/// 2. **União (union)**: Se o CT-e A cita o CT-e B (ex: subcontratação), eles dão as mãos.
///    O líder do grupo de A passa a reconhecer o líder do grupo de B.
/// 3. **Transitividade**: Se o CT-e B cita o CT-e C (ex: complemento), o grupo de B se une ao de C.
///    Automaticamente, A, B e C agora pertencem **à mesma roda**.
/// 4. **Busca (find) com Compressão de Caminho**: Quando perguntamos a qualquer membro quem é seu líder supremo,
///    ele responde e já ajusta seu ponteiro para apontar diretamente para o líder principal, eliminando intermediários.
///
/// ---
///
/// ## O Problema do Grafo Fiscal Brasileiro
///
/// Na legislação de transportes da SEFAZ, as relações entre CT-es formam uma **Relação de Equivalência**:
/// * **Simetria:** Se o documento A referencia B, então B está relacionado a A.
/// * **Transitividade:** Se A relaciona com B, e B com C, então A relaciona com C.
///
/// ```text
/// Entrada Bruta (Arestas no XML):
///    CTe_1 ────> CTe_2
///    CTe_2 ────> CTe_3
///    CTe_4 ────> CTe_5
///
/// Componentes Conexas Identificadas pelo DSU:
///    ┌────────────────────────────────┐      ┌─────────────────────────┐
///    │  Grupo 1: {CTe_1, CTe_2, CTe_3}│      │ Grupo 2: {CTe_4, CTe_5} │
///    └────────────────────────────────┘      └─────────────────────────┘
/// ```
///
/// ---
///
/// ## Mecânica Interna (Zero Alocações na Heap)
///
/// 1. **Indexação Inteira**: Cada chave textual de 44 dígitos é mapeada para um índice 0, 1, 2, ..., N-1.
/// 2. **Vetor de Pais (parent: Vec<usize>)**:
///
/// ```text
/// Estado Inicial (Cada nó é pai de si mesmo):
/// Índice:  [  0,   1,   2,   3,   4 ]
/// Parent:  [  0,   1,   2,   3,   4 ]
///
/// union(0, 1) -> Nó 0 passa a seguir o líder 1:
/// Parent:  [  1,   1,   2,   3,   4 ]
///
/// union(1, 2) -> Líder 1 passa a seguir o líder 2:
/// Parent:  [  1,   2,   2,   3,   4 ]
///
/// find(0) com Compressão -> Nó 0 descobre que o líder final é 2 e atualiza seu ponteiro direto:
/// Parent:  [  2,   2,   2,   3,   4 ]  <-- Consulta instantânea O(1)!
/// ```
#[derive(Debug, Clone)]
struct DisjointSet {
    /// Aponta para o índice do elemento pai/representante do conjunto.
    parent: Vec<usize>,
    /// Altura aproximada da árvore do conjunto (usado na união por rank).
    rank: Vec<u8>,
}

impl DisjointSet {
    /// Cria uma nova floresta de conjuntos disjuntos onde cada elemento de 0..size é seu próprio líder.
    #[inline]
    fn new(size: usize) -> Self {
        Self {
            parent: (0..size).collect(),
            rank: vec![0; size],
        }
    }

    /// Encontra o representante supremo (raiz) do conjunto ao qual o elemento `i` pertence.
    ///
    /// Aplica a técnica de **Compressão de Caminho (Path Compression)** de forma iterativa,
    /// fazendo com que todos os nós visitados apontem diretamente para a raiz.
    ///
    /// # Complexidade
    /// Amortizada O(alfa(N)), onde alfa é a Função Inversa de Ackermann (na prática, alfa(N) <= 4, ou seja, quase O(1)).
    fn find(&mut self, i: usize) -> usize {
        let mut root = i;
        while root != self.parent[root] {
            root = self.parent[root];
        }

        // Compressão de caminho iterativa (segura contra estouro de pilha / stack overflow)
        let mut curr = i;
        while curr != root {
            let next = self.parent[curr];
            self.parent[curr] = root;
            curr = next;
        }

        root
    }

    /// Une os conjuntos que contêm os elementos `i` e `j`.
    ///
    /// Utiliza a heurística de **União por Rank (Union by Rank)**, acoplando a árvore de menor
    /// profundidade sob a raiz da árvore mais profunda, mantendo a estrutura equilibrada.
    #[inline]
    fn union(&mut self, i: usize, j: usize) {
        let root_i = self.find(i);
        let root_j = self.find(j);

        if root_i != root_j {
            match self.rank[root_i].cmp(&self.rank[root_j]) {
                Ordering::Less => self.parent[root_i] = root_j,
                Ordering::Greater => self.parent[root_j] = root_i,
                Ordering::Equal => {
                    self.parent[root_i] = root_j;
                    self.rank[root_j] = self.rank[root_j].saturating_add(1);
                }
            }
        }
    }
}

/// Extensão para expansão e inversão de grafos de relacionamento fiscal.
pub trait GraphExtension {
    /// Conta recursivamente o número total de vértices e arestas no grafo.
    fn count_recursively(&self) -> usize;

    /// Expande as relações de equivalência entre Conhecimentos de Transporte (CT-es).
    ///
    /// Calcula o fecho simétrico e transitivo do grafo de documentos fiscais.
    ///
    /// # Conceito do Negócio Fiscal
    /// No transporte de cargas, documentos complementares, redespachos e subcontratações
    /// formam cadeias de prestação de serviço. Esta função garante que qualquer CT-e
    /// pertencente à mesma cadeia reconheça todos os outros documentos correlacionados.
    ///
    /// # Regras Aplicadas:
    /// 1. **Anti-Reflexividade:** Um CT-e nunca referencia a si próprio (evita auto-laços).
    /// 2. **Simetria:** Se o CT-e A referencia o CT-e B, então B passa a referenciar A mutuamente.
    /// 3. **Transitividade:** Se o CT-e A referencia B, e B referencia C, então A referencia C (e vice-versa).
    ///
    /// # Critério de Parada (Ponto Fixo):
    /// O laço `loop` executa rodadas sucessivas de expansão. Ele encerra automaticamente
    /// quando uma rodada inteira não descobre nenhuma nova conexão (`count_keys == 0`).
    ///
    /// # Exemplos
    ///
    /// ```rust
    /// use std::collections::{HashMap, HashSet};
    /// use read_xml::GraphExtension;
    ///
    /// let mut grafo = HashMap::from([
    ///     ("cte_a".to_string(), HashSet::from(["cte_b".to_string()])),
    ///     ("cte_b".to_string(), HashSet::from(["cte_c".to_string()])),
    /// ]);
    ///
    /// grafo.expand_ctes(false);
    ///
    /// // Todos os nós agora apontam mutuamente entre si:
    /// assert!(grafo.get("cte_a").unwrap().contains("cte_c"));
    /// assert!(grafo.get("cte_c").unwrap().contains("cte_a"));
    /// ```
    fn expand_ctes(&mut self, verbose: bool);

    fn expand_ctes_old(&mut self, verbose: bool);

    /// Propaga as Notas Fiscais Eletrônicas (NF-es) entre todos os CT-es correlacionados.
    ///
    /// Se um `CTe_A` transporta a `NFe_1` e está vinculado ao `CTe_B` (via subcontratação ou
    /// complemento), o `CTe_B` herdará automaticamente a referência à `NFe_1`.
    ///
    /// # Parâmetros
    /// * `cte_ctes`: Grafo previamente expandido contendo as correlações entre CT-es.
    fn expand_nfes(&mut self, cte_ctes: &Self);

    /// Filtra as NF-es mantendo apenas aquelas confirmadas como ativas/válidas no lote.
    fn filtrar_nfes_validos(&self, nfe_info: &BTreeMap<KeyDoc, Vec<InfoNfe>>) -> Self;

    /// Filtra os CT-es mantendo apenas aqueles confirmados como ativos/válidos no lote.
    fn filtrar_ctes_validos(&self, cte_info: &BTreeMap<KeyDoc, Vec<InfoCte>>) -> Self;

    /// Inverte a direção das arestas do grafo bipartido (`CT-e -> NF-e` para `NF-e -> CT-e`).
    ///
    /// Mapeia para cada Nota Fiscal Eletrônica quais foram todos os Conhecimentos de Transporte
    /// que participaram de sua movimentação física.
    ///
    /// # Exemplos
    ///
    /// ```rust
    /// use std::collections::{HashMap, HashSet};
    /// use read_xml::GraphExtension;
    ///
    /// let cte_nfes = HashMap::from([
    ///     ("cte_1".to_string(), HashSet::from(["nfe_a".to_string(), "nfe_b".to_string()])),
    /// ]);
    ///
    /// let nfe_ctes = cte_nfes.get_nfe_ctes();
    ///
    /// assert!(nfe_ctes.get("nfe_a").unwrap().contains("cte_1"));
    /// assert!(nfe_ctes.get("nfe_b").unwrap().contains("cte_1"));
    /// ```
    fn get_nfe_ctes(&self) -> Self;
}

impl GraphExtension for HashMap<String, HashSet<String>> {
    #[inline]
    fn count_recursively(&self) -> usize {
        self.len() + self.values().map(HashSet::len).sum::<usize>()
    }

    /// Executa o cálculo de componentes conexas sobre os relacionamentos entre CT-es em tempo linear O(V + E).
    ///
    /// Garante que todos os CT-es pertencentes à mesma cadeia operacional (subcontratação, redespacho,
    /// complementares) conheçam todos os seus pares mutuamente.
    fn expand_ctes(&mut self, verbose: bool) {
        if self.is_empty() {
            return;
        }

        // ---------------------------------------------------------------------
        // ETAPA 1: INDEXAÇÃO INTEIRA (Zero alocações na Heap para a lógica)
        // ---------------------------------------------------------------------
        // Mapeia strings para inteiros 0..N, permitindo operações contíguas em cache L1/L2
        let mut key_to_id: HashMap<&str, usize> = HashMap::with_capacity(self.len() * 2);
        let mut id_to_key: Vec<&str> = Vec::with_capacity(self.len() * 2);

        for (k, vizinhos) in self.iter() {
            if !key_to_id.contains_key(k.as_str()) {
                key_to_id.insert(k.as_str(), id_to_key.len());
                id_to_key.push(k.as_str());
            }
            for v in vizinhos {
                if !key_to_id.contains_key(v.as_str()) {
                    key_to_id.insert(v.as_str(), id_to_key.len());
                    id_to_key.push(v.as_str());
                }
            }
        }

        let num_nodes = id_to_key.len();
        let mut dsu = DisjointSet::new(num_nodes);

        // ---------------------------------------------------------------------
        // ETAPA 2: UNIÃO EM UMA ÚNICA PASSADA O(E)
        // ---------------------------------------------------------------------
        // Processa cada relacionamento declarado no XML conectando os conjuntos
        for (k, vizinhos) in self.iter() {
            let u = key_to_id[k.as_str()];
            for v_str in vizinhos {
                let v = key_to_id[v_str.as_str()];
                dsu.union(u, v);
            }
        }

        // ---------------------------------------------------------------------
        // ETAPA 3: AGRUPAMENTO POR COMPONENTES CONEXAS
        // ---------------------------------------------------------------------
        // Agrupa todos os documentos que compartilham o mesmo líder supremo (raiz)
        let mut componentes: HashMap<usize, Vec<&str>> = HashMap::new();

        for (id, &key) in id_to_key.iter().enumerate() {
            let root = dsu.find(id);
            componentes.entry(root).or_default().push(key);
        }

        if verbose {
            println!("Total de documentos indexados: {num_nodes}");
            println!(
                "Total de grupos conexos independentes: {}",
                componentes.len()
            );
        }

        // ---------------------------------------------------------------------
        // ETAPA 4: RECONSTRUÇÃO DETERMINÍSTICA DO GRAFO (Sem conflito de borrow)
        // ---------------------------------------------------------------------
        let mut novo_grafo: HashMap<String, HashSet<String>> = HashMap::with_capacity(num_nodes);

        for (_root, membros) in componentes {
            // Documentos isolados sem relacionamentos com terceiros não geram conexões
            if membros.len() <= 1 {
                continue;
            }

            let grupo_set: HashSet<String> = membros.iter().map(|&s| s.to_string()).collect();

            for &membro in &membros {
                let mut vizinhos = grupo_set.clone();
                vizinhos.remove(membro); // Regra Anti-reflexiva: um CT-e não referencia a si mesmo
                novo_grafo.insert(membro.to_string(), vizinhos);
            }
        }

        // Substituição final atômica: libera com segurança os empréstimos imutáveis de &str
        *self = novo_grafo;
    }

    fn expand_ctes_old(&mut self, verbose: bool) {
        // Laço de ponto fixo: repete rodadas de expansão até o grafo estabilizar
        loop {
            // Contador de novas arestas inseridas NESTA rodada específica
            let mut novas_arestas = 0;

            // Fila de trabalho (Worklist): armazena em um vetor contíguo (memória contínua e rápida)
            // os pares (Origem, Destino) que precisam ser inseridos no mapa principal.
            // Isso evita conflitos com o Borrow Checker durante a leitura imutável do `self`.
            let mut worklist: Vec<(String, String)> = Vec::new();

            // -----------------------------------------------------------------
            // FASE 1: DESCOBERTA DE NOVAS RELAÇÕES (simétricas e transitivas)
            // -----------------------------------------------------------------
            for (cte_a, vizinhos_de_a) in self.iter() {
                for cte_b in vizinhos_de_a {
                    // Evita auto-referência: um CT-e não deve apontar para si mesmo
                    if cte_a == cte_b {
                        continue;
                    }
                    // Simetria: Se A -> B, então B -> A
                    if !self
                        .get(cte_b)
                        .is_some_and(|vizinhos_b| vizinhos_b.contains(cte_a))
                    {
                        worklist.push((cte_b.clone(), cte_a.clone()));
                    }

                    // Transitividade: Se A -> B e B -> C, então A -> C
                    if let Some(vizinhos_de_b) = self.get(cte_b) {
                        for cte_c in vizinhos_de_b {
                            // Garante que não criaremos auto-laço (cte_a -> cte_a)
                            if cte_a != cte_c && !vizinhos_de_a.contains(cte_c) {
                                worklist.push((cte_a.clone(), cte_c.clone()));
                            }
                        }
                    }
                }
            }

            // -----------------------------------------------------------------
            // FASE 2: APLICAÇÃO E MEDIÇÃO DE NOVIDADES (Mutação Controlada)
            // -----------------------------------------------------------------
            // Aplica no grafo principal todas as arestas descobertas na Fase 1.
            for (origem, destino) in worklist {
                // Por segurança redundante, assegurar exclusão de auto-referência.
                if origem == destino {
                    continue;
                }
                // `HashSet::insert()` retorna `true` se o valor era INÉDITO no conjunto,
                // e `false` se o elemento já existia previamente.
                // Usamos esse retorno nativo do Rust para saber se o grafo realmente cresceu!
                if self.entry(origem).or_default().insert(destino) {
                    novas_arestas += 1;
                }
            }

            // Diagnóstico visual opcional para depuração e rastreamento de lotes
            if verbose && novas_arestas > 0 {
                println!("--- Estado Atual da Convergência de CT-es ---");
                for (k, chaves) in self.iter() {
                    println!(" {k}: {chaves:?}");
                }
                println!("Novas conexões adicionadas nesta rodada: {novas_arestas}\n");
            }

            // -----------------------------------------------------------------
            // FASE 3: VERIFICAÇÃO DO PONTO FIXO (Critério de Parada)
            // -----------------------------------------------------------------
            // Se nenhuma nova aresta foi inserida nesta rodada (novas_arestas == 0),
            // significa que todas as conexões transitivas e simétricas possíveis já
            // foram descobertas. O grafo atingiu a estabilidade final.
            if novas_arestas == 0 {
                break;
            }
        }
    }

    fn expand_nfes(&mut self, cte_ctes: &Self) {
        for (cte_a, nfes) in self.clone() {
            if let Some(ctes_relacionados) = cte_ctes.get(&cte_a) {
                for cte_relacionado in ctes_relacionados {
                    self.entry(cte_relacionado.clone())
                        .or_default()
                        .extend(nfes.clone());
                }
            }
        }
    }

    fn filtrar_nfes_validos(&self, nfe_info: &BTreeMap<KeyDoc, Vec<InfoNfe>>) -> Self {
        self.par_iter()
            .map(|(cte, nfes)| {
                let docs_validos: HashSet<String> = nfes
                    .iter()
                    .filter(|&nfe| {
                        let chave_valida = KeyDoc::new(nfe, true);
                        nfe_info.contains_key(&chave_valida)
                    })
                    .cloned()
                    .collect();

                (cte.clone(), docs_validos)
            })
            .filter(|(_, docs_validos)| !docs_validos.is_empty())
            .collect()
    }

    fn filtrar_ctes_validos(&self, cte_info: &BTreeMap<KeyDoc, Vec<InfoCte>>) -> Self {
        self.par_iter()
            .map(|(cte, ctes)| {
                let docs_validos: HashSet<String> = ctes
                    .iter()
                    .filter(|&cte| {
                        let chave_valida = KeyDoc::new(cte, true);
                        cte_info.contains_key(&chave_valida)
                    })
                    .cloned()
                    .collect();

                (cte.clone(), docs_validos)
            })
            .filter(|(_, docs_validos)| !docs_validos.is_empty())
            .collect()
    }

    /// Get nfe_ctes from cte_nfes
    ///
    /// Inverter a ordem: (cte, nfes) -> (nfe, ctes)
    ///
    /// HashMap<chave_nfe, HashSet<chave_cte>>
    fn get_nfe_ctes(&self) -> Self {
        let mut nfe_ctes: HashMap<String, HashSet<String>> = HashMap::new();

        for (cte, nfes) in self {
            for nfe in nfes {
                nfe_ctes
                    .entry(nfe.clone())
                    // If there's no entry for the key cte, create a new Vec and return a mutable ref to it
                    .or_default()
                    // and insert the item onto the Vec
                    .insert(cte.clone());
            }
        }

        nfe_ctes
    }
}

//----------------------------------------------------------------------------//
//                                   Tests                                    //
//----------------------------------------------------------------------------//

// cargo test -- --help
// cargo test -- --nocapture
// cargo test -- --show-output

/// Run tests with:
/// cargo test -- --show-output tests_graph
#[cfg(test)]
mod tests_graph {
    use crate::XmlParserResult;
    use std::collections::BTreeSet;

    use super::*;

    // cargo test -- --help
    // cargo test -- --show-output
    // cargo test -- --show-output multiple_values

    #[test]
    /// cargo test -- --show-output hashset_btreeset
    fn hashset_btreeset() {
        // Obter lista com elementos únicos não necessariamente ordenados
        let hashset = HashSet::from([3, 1, 2, 3, 2]);
        println!("hashset: {hashset:?}");

        // Obter lista com elementos únicos ordenados
        let btreeset = BTreeSet::from([3, 1, 2, 3, 2]);
        println!("btreeset: {btreeset:?}");

        assert_eq!(hashset, HashSet::from([1, 2, 3]));
        assert_eq!(btreeset, BTreeSet::from([1, 2, 3]));
    }

    /**
        cte_a: ["cte_1", "cte_2", "cte_3"]

        cte_complementar: ["cte_3", "cte_4"]

        cte_c: ["cte_5", "cte_6"]

        cte_3 ∈ cte_a && cte_3 ∈ cte_complementar

        Portanto:
        cte_a e cte_complementar são correlacionados pois contém "cte_3" em comum.

        `cargo test -- --show-output correlacionar_ctes`
    */
    #[test]
    fn correlacionar_ctes() -> XmlParserResult<()> {
        // Start
        let array_a = ["cte_1", "cte_2", "cte_3"];
        let array_b = ["cte_3", "cte_4"];
        let array_c = ["cte_5", "cte_6"];
        let array_d = ["cte_7"];

        let mut cte_ctes: HashMap<String, HashSet<String>> = HashMap::from([
            (
                "cte_a".to_string(),
                HashSet::from(array_a.map(String::from)),
            ),
            (
                "cte_complementar".to_string(),
                HashSet::from(array_b.map(String::from)),
            ),
            (
                "cte_c".to_string(),
                HashSet::from(array_c.map(String::from)),
            ),
            (
                "cte_d".to_string(),
                HashSet::from(array_d.map(String::from)),
            ),
        ]);

        println!("cte_ctes: [Start]");
        cte_ctes
            .iter()
            .for_each(|(k, chaves)| println!("{k}: {chaves:?}"));
        println!("cte_ctes.len(): {}\n", cte_ctes.len());

        cte_ctes.expand_ctes(true);

        println!("cte_ctes: [Final]");
        cte_ctes
            .iter()
            .for_each(|(k, chaves)| println!("{k}: {chaves:?}"));
        println!("cte_ctes.len(): {}\n", cte_ctes.len());

        // Final
        let array_1 = ["cte_2", "cte_3", "cte_4", "cte_a", "cte_complementar"];
        let array_2 = ["cte_1", "cte_3", "cte_4", "cte_a", "cte_complementar"];
        let array_3 = ["cte_1", "cte_2", "cte_4", "cte_a", "cte_complementar"];
        let array_4 = ["cte_1", "cte_2", "cte_3", "cte_a", "cte_complementar"];
        let array_5 = ["cte_6", "cte_c"];
        let array_6 = ["cte_5", "cte_c"];
        let array_7 = ["cte_d"];
        let array_a = ["cte_1", "cte_2", "cte_3", "cte_4", "cte_complementar"];
        let array_b = ["cte_1", "cte_2", "cte_3", "cte_4", "cte_a"];
        let array_c = ["cte_5", "cte_6"];
        let array_d = ["cte_7"];

        assert_eq!(
            cte_ctes,
            HashMap::from([
                (
                    "cte_1".to_string(),
                    HashSet::from(array_1.map(String::from))
                ),
                (
                    "cte_2".to_string(),
                    HashSet::from(array_2.map(String::from))
                ),
                (
                    "cte_3".to_string(),
                    HashSet::from(array_3.map(String::from))
                ),
                (
                    "cte_4".to_string(),
                    HashSet::from(array_4.map(String::from))
                ),
                (
                    "cte_5".to_string(),
                    HashSet::from(array_5.map(String::from))
                ),
                (
                    "cte_6".to_string(),
                    HashSet::from(array_6.map(String::from))
                ),
                (
                    "cte_7".to_string(),
                    HashSet::from(array_7.map(String::from))
                ),
                (
                    "cte_a".to_string(),
                    HashSet::from(array_a.map(String::from))
                ),
                (
                    "cte_complementar".to_string(),
                    HashSet::from(array_b.map(String::from))
                ),
                (
                    "cte_c".to_string(),
                    HashSet::from(array_c.map(String::from))
                ),
                (
                    "cte_d".to_string(),
                    HashSet::from(array_d.map(String::from))
                ),
            ])
        );

        Ok(())
    }

    /// `cargo test -- --show-output expand_cte_nfes`
    #[test]
    fn expand_cte_nfes() -> XmlParserResult<()> {
        // Start
        let array_a = ["cte_1", "cte_2", "cte_3"];
        let array_b = ["cte_3", "cte_4"];
        let array_c = ["cte_5", "cte_6"];
        let array_d = ["cte_7"];

        let mut cte_ctes: HashMap<String, HashSet<String>> = HashMap::from([
            (
                "cte_a".to_string(),
                HashSet::from(array_a.map(String::from)),
            ),
            (
                "cte_complementar".to_string(),
                HashSet::from(array_b.map(String::from)),
            ),
            (
                "cte_c".to_string(),
                HashSet::from(array_c.map(String::from)),
            ),
            (
                "cte_d".to_string(),
                HashSet::from(array_d.map(String::from)),
            ),
        ]);

        let array_1 = ["nfe_1", "nfe_2"];
        let array_2 = ["nfe_2", "nfe_3"];
        let array_3 = ["nfe_4"];

        let mut cte_nfes: HashMap<String, HashSet<String>> = HashMap::from([
            (
                "cte_1".to_string(),
                HashSet::from(array_1.map(String::from)),
            ),
            (
                "cte_3".to_string(),
                HashSet::from(array_2.map(String::from)),
            ),
            (
                "cte_8".to_string(),
                HashSet::from(array_3.map(String::from)),
            ),
        ]);

        cte_ctes.expand_ctes(false);

        cte_nfes.expand_nfes(&cte_ctes);

        println!("cte_nfes:");
        cte_nfes
            .iter()
            .for_each(|(cte, nfes)| println!("{cte}: {:>2} {nfes:?}", nfes.len()));
        println!("cte_nfes.len(): {}\n", cte_nfes.len());

        let array_x = ["nfe_1", "nfe_2", "nfe_3"];
        let array_y = ["nfe_4"];

        // Final
        assert_eq!(
            cte_nfes,
            HashMap::from([
                (
                    "cte_1".to_string(),
                    HashSet::from(array_x.map(String::from))
                ),
                (
                    "cte_2".to_string(),
                    HashSet::from(array_x.map(String::from))
                ),
                (
                    "cte_3".to_string(),
                    HashSet::from(array_x.map(String::from))
                ),
                (
                    "cte_4".to_string(),
                    HashSet::from(array_x.map(String::from))
                ),
                (
                    "cte_8".to_string(),
                    HashSet::from(array_y.map(String::from))
                ),
                (
                    "cte_a".to_string(),
                    HashSet::from(array_x.map(String::from))
                ),
                (
                    "cte_complementar".to_string(),
                    HashSet::from(array_x.map(String::from))
                ),
            ])
        );

        Ok(())
    }

    #[test]
    fn test_expand_ctes_dsu_equivalencia() {
        let mut grafo = HashMap::from([
            ("cte_1".to_string(), HashSet::from(["cte_2".to_string()])),
            ("cte_2".to_string(), HashSet::from(["cte_3".to_string()])),
            ("cte_4".to_string(), HashSet::from(["cte_5".to_string()])),
        ]);

        grafo.expand_ctes(true);

        // Grupo 1: {1, 2, 3} todos enxergam todos entre si
        assert_eq!(
            grafo.get("cte_1").unwrap(),
            &HashSet::from(["cte_2".to_string(), "cte_3".to_string()])
        );
        assert_eq!(
            grafo.get("cte_3").unwrap(),
            &HashSet::from(["cte_1".to_string(), "cte_2".to_string()])
        );

        // Grupo 2: {4, 5}
        assert_eq!(
            grafo.get("cte_4").unwrap(),
            &HashSet::from(["cte_5".to_string()])
        );
        assert_eq!(
            grafo.get("cte_5").unwrap(),
            &HashSet::from(["cte_4".to_string()])
        );
    }
}

/// Run tests with:
/// cargo test -- --show-output tests_complexos_grafo
#[cfg(test)]
mod tests_complexos_grafo {
    use super::*;
    use std::collections::{HashMap, HashSet};

    /// Função auxiliar para validar se um grupo de membros forma um subgrafo completo (clique)
    /// onde cada nó enxerga exatamente todos os outros integrantes do seu grupo e nenhum nó externo.
    fn validar_grupo_conexo(grafo: &HashMap<String, HashSet<String>>, membros: &[&str]) {
        for &membro in membros {
            let vizinhos = grafo.get(membro).unwrap_or_else(|| {
                panic!(
                    "O documento '{}' deveria estar presente no grafo final.",
                    membro
                )
            });

            // 1. O número de vizinhos deve ser exatamente o tamanho do grupo menos ele próprio
            assert_eq!(
                vizinhos.len(),
                membros.len() - 1,
                "Quantidade incorreta de vizinhos para '{}'. Esperado: {}, Obtido: {}",
                membro,
                membros.len() - 1,
                vizinhos.len()
            );

            // 2. Anti-reflexividade: não pode conter a si mesmo
            assert!(
                !vizinhos.contains(membro),
                "Violação de anti-reflexividade: '{}' contém a si mesmo.",
                membro
            );

            // 3. Deve conter todos os outros integrantes do grupo
            for &outro in membros {
                if outro != membro {
                    assert!(
                        vizinhos.contains(outro),
                        "Violação de transitividade: '{}' não encontrou '{}' no seu grupo.",
                        membro,
                        outro
                    );
                }
            }
        }
    }

    #[test]
    fn test_expand_ctes_grupos_heterogeneos() {
        let mut grafo: HashMap<String, HashSet<String>> = HashMap::new();

        // ---------------------------------------------------------------------
        // GRUPO 1: 1 Elemento Isolado (Singleton)
        // CT-e sem qualquer referência a outro documento.
        // ---------------------------------------------------------------------
        grafo.insert("cte_iso_01".to_string(), HashSet::new());

        // ---------------------------------------------------------------------
        // GRUPO 2: 2 Elementos (Par Simples A -> B)
        // Subcontratação direta simples.
        // ---------------------------------------------------------------------
        grafo.insert(
            "cte_par_01".to_string(),
            HashSet::from(["cte_par_02".to_string()]),
        );

        // ---------------------------------------------------------------------
        // GRUPO 3: 3 Elementos (Cadeia Linear A -> B -> C)
        // Transporte original -> Redespacho -> Complementar.
        // ---------------------------------------------------------------------
        grafo.insert(
            "cte_trio_01".to_string(),
            HashSet::from(["cte_trio_02".to_string()]),
        );
        grafo.insert(
            "cte_trio_02".to_string(),
            HashSet::from(["cte_trio_03".to_string()]),
        );

        // ---------------------------------------------------------------------
        // GRUPO 4: 5 Elementos (Topologia em Estrela e Ciclo Fechado)
        // q1 conecta com q2 e q3; q3 conecta com q4; q4 com q5; q5 fecha ciclo em q1.
        // ---------------------------------------------------------------------
        grafo.insert(
            "cte_quin_01".to_string(),
            HashSet::from(["cte_quin_02".to_string(), "cte_quin_03".to_string()]),
        );
        grafo.insert(
            "cte_quin_03".to_string(),
            HashSet::from(["cte_quin_04".to_string()]),
        );
        grafo.insert(
            "cte_quin_04".to_string(),
            HashSet::from(["cte_quin_05".to_string()]),
        );
        grafo.insert(
            "cte_quin_05".to_string(),
            HashSet::from(["cte_quin_01".to_string()]),
        );

        // ---------------------------------------------------------------------
        // GRUPO 5: 10 Elementos (Rede Complexa com 2 Ramos Unidos por uma Ponte)
        // Ramo A: d01 -> d02 -> (d03, d04) -> d05
        // Ramo B: d07 -> d08 -> d09 -> d10
        // Ponte que une os dois ramos: d05 -> d06 -> d07
        // ---------------------------------------------------------------------
        grafo.insert(
            "cte_dec_01".to_string(),
            HashSet::from(["cte_dec_02".to_string()]),
        );
        grafo.insert(
            "cte_dec_02".to_string(),
            HashSet::from(["cte_dec_03".to_string(), "cte_dec_04".to_string()]),
        );
        grafo.insert(
            "cte_dec_04".to_string(),
            HashSet::from(["cte_dec_05".to_string()]),
        );
        // Ponte entre ramos
        grafo.insert(
            "cte_dec_05".to_string(),
            HashSet::from(["cte_dec_06".to_string()]),
        );
        grafo.insert(
            "cte_dec_06".to_string(),
            HashSet::from(["cte_dec_07".to_string()]),
        );
        // Ramo B
        grafo.insert(
            "cte_dec_07".to_string(),
            HashSet::from(["cte_dec_08".to_string()]),
        );
        grafo.insert(
            "cte_dec_08".to_string(),
            HashSet::from(["cte_dec_09".to_string()]),
        );
        grafo.insert(
            "cte_dec_09".to_string(),
            HashSet::from(["cte_dec_10".to_string()]),
        );

        // ---------------------------------------------------------------------
        // EXECUÇÃO DO ALGORITMO DSU
        // ---------------------------------------------------------------------
        grafo.expand_ctes(true);

        // ---------------------------------------------------------------------
        // VALIDAÇÕES DE INTEGRIDADE E ISOLAMENTO
        // ---------------------------------------------------------------------

        // 1. Validação do Grupo Isolado (Tamanho 1):
        // Documentos sem conexões externas não devem figurar com vizinhos.
        assert!(
            !grafo.contains_key("cte_iso_01") || grafo.get("cte_iso_01").unwrap().is_empty(),
            "O elemento isolado 'cte_iso_01' nao deveria possuir conexoes."
        );

        // 2. Validação do Grupo de 2 Elementos:
        let membros_par = ["cte_par_01", "cte_par_02"];
        validar_grupo_conexo(&grafo, &membros_par);

        // 3. Validação do Grupo de 3 Elementos:
        let membros_trio = ["cte_trio_01", "cte_trio_02", "cte_trio_03"];
        validar_grupo_conexo(&grafo, &membros_trio);

        // 4. Validação do Grupo de 5 Elementos:
        let membros_quin = [
            "cte_quin_01",
            "cte_quin_02",
            "cte_quin_03",
            "cte_quin_04",
            "cte_quin_05",
        ];
        validar_grupo_conexo(&grafo, &membros_quin);

        // 5. Validação do Grupo de 10 Elementos:
        let membros_dec = [
            "cte_dec_01",
            "cte_dec_02",
            "cte_dec_03",
            "cte_dec_04",
            "cte_dec_05",
            "cte_dec_06",
            "cte_dec_07",
            "cte_dec_08",
            "cte_dec_09",
            "cte_dec_10",
        ];
        validar_grupo_conexo(&grafo, &membros_dec);

        // ---------------------------------------------------------------------
        // VALIDAÇÃO DE NÃO-VAZAMENTO ENTRE GRUPOS INDEPENDENTES
        // Nenhum CT-e do grupo de 10 pode enxergar membros do grupo de 5 ou 3.
        // ---------------------------------------------------------------------
        let vizinhos_dec_01 = grafo.get("cte_dec_01").unwrap();
        assert!(!vizinhos_dec_01.contains("cte_quin_01"));
        assert!(!vizinhos_dec_01.contains("cte_trio_01"));
        assert!(!vizinhos_dec_01.contains("cte_par_01"));

        let vizinhos_quin_01 = grafo.get("cte_quin_01").unwrap();
        assert!(!vizinhos_quin_01.contains("cte_dec_10"));
        assert!(!vizinhos_quin_01.contains("cte_trio_03"));

        // Total de nós registrados no mapa final (2 + 3 + 5 + 10 = 20 nós ativos)
        assert_eq!(grafo.len(), 20);
    }
}

/// Run tests with:
/// cargo test -- --show-output tests_graphos_comparativos
#[cfg(test)]
mod tests_graphos_comparativos {
    use super::*;
    use std::collections::{HashMap, HashSet};

    /// Helper para inserir conexões direcionadas iniciais no grafo de teste.
    fn conectar(grafo: &mut HashMap<String, HashSet<String>>, de: &str, para: &str) {
        grafo
            .entry(de.to_string())
            .or_default()
            .insert(para.to_string());
    }

    #[test]
    fn test_comparativo_dsu_vs_ponto_fixo() {
        let mut grafo_original: HashMap<String, HashSet<String>> = HashMap::new();

        // ---------------------------------------------------------------------
        // GRUPO 1: Par Simples (2 elementos)
        // Subcontratação direta: p1 -> p2
        // ---------------------------------------------------------------------
        conectar(&mut grafo_original, "cte_p1", "cte_p2");

        /*
        Entrada (Galho Direcionado):
            [cte_p1] ───────────────> [cte_p2]

        Saída (Grafo Completo / Simétrico):
            [cte_p1] <══════════════> [cte_p2]
            * Cada nó possui exatamente 1 vizinho.
        */

        // ---------------------------------------------------------------------
        // GRUPO 2: Cadeia Linear (3 elementos)
        // Transporte original -> Redespacho -> Complementar: t1 -> t2 -> t3
        // ---------------------------------------------------------------------
        conectar(&mut grafo_original, "cte_t1", "cte_t2");
        conectar(&mut grafo_original, "cte_t2", "cte_t3");

        /*
        Entrada (Galho em Linha):
            [cte_t1] ───────> [cte_t2] ───────> [cte_t3]

        Saída (Árvore Fechada / Triângulo Completo):
                    [cte_t1]
                    ▲      ▲
                   /        \
                  ▼          ▼
              [cte_t2] <════> [cte_t3]
              * Cada nó conhece os outros 2 integrantes.
         */

        // ---------------------------------------------------------------------
        // GRUPO 3: Topologia com Ciclos e Ramificações (5 elementos)
        // q1 conecta com q2 e q3; q3 conecta com q4; q4 com q5; q5 fecha em q2
        // ---------------------------------------------------------------------
        conectar(&mut grafo_original, "cte_q1", "cte_q2");
        conectar(&mut grafo_original, "cte_q1", "cte_q3");
        conectar(&mut grafo_original, "cte_q3", "cte_q4");
        conectar(&mut grafo_original, "cte_q4", "cte_q5");
        conectar(&mut grafo_original, "cte_q5", "cte_q2");

        /*
        Entrada (Bifurcação e Ciclo):
                        ┌───────────> [cte_q2] <──────────┐
                        │                                 │
           [cte_q1] ────┤                                 │ (retorno)
                        │                                 │
                        └───────────> [cte_q3]            │
                                        │                 │
                                        ▼                 │
                                     [cte_q4] ───────> [cte_q5]

        Saída (Rede de 5 Nós):
            Todos os 5 nós (q1, q2, q3, q4, q5) se conectam mutuamente.
            * Cada nó possui exatamente 4 vizinhos no mapa final.
        */

        // ---------------------------------------------------------------------
        // GRUPO 4: Topologia em Estrela (8 elementos)
        // Um CT-e central concentrador referenciado por outros 7 CT-es
        // ---------------------------------------------------------------------
        for i in 1..=7 {
            let satelite = format!("cte_est_{:02}", i);
            conectar(&mut grafo_original, "cte_est_hub", &satelite);
        }

        /*
        Um CT-e central (Hub / Matriz) ao qual 7 outros CT-es satélites estão vinculados.

        Entrada (Estrutura em Estrela):
                            [cte_est_01]
                            /      |     \
                  [cte_est_02]     |      [cte_est_03]
                        \          |          /
                        ───> [cte_est_hub] <───
                        /     /    |    \     \
                [cte_est_04] [cte_est_05] [cte_est_06] [cte_est_07]

        Saída (Componente Conexa Completa):
            O hub e todos os 7 satélites formam um grupo onde cada um enxerga 7 vizinhos.
        */

        // ---------------------------------------------------------------------
        // GRUPO 5: Rede Densa de Subcontratação Intermediária (12 elementos)
        // Múltiplos caminhos paralelos, cruzamentos e pontes
        // ---------------------------------------------------------------------
        // Ramo 1
        conectar(&mut grafo_original, "cte_g01", "cte_g02");
        conectar(&mut grafo_original, "cte_g02", "cte_g03");
        conectar(&mut grafo_original, "cte_g02", "cte_g04");
        conectar(&mut grafo_original, "cte_g03", "cte_g05");
        conectar(&mut grafo_original, "cte_g04", "cte_g05");

        // Ponte conectando os Ramos
        conectar(&mut grafo_original, "cte_g05", "cte_g06");
        conectar(&mut grafo_original, "cte_g06", "cte_g07");

        // Ramo 2
        conectar(&mut grafo_original, "cte_g07", "cte_g08");
        conectar(&mut grafo_original, "cte_g07", "cte_g09");
        conectar(&mut grafo_original, "cte_g08", "cte_g10");
        conectar(&mut grafo_original, "cte_g09", "cte_g11");
        conectar(&mut grafo_original, "cte_g10", "cte_g12");
        conectar(&mut grafo_original, "cte_g11", "cte_g12");

        /*
        Dois grandes blocos de fretes conectados no meio por uma ponte (tronco) de subcontratação.

        RAMO ESQUERDO (Coleta)          TRONCO / PONTE                 RAMO DIREITO (Entrega)
        (5 CT-es)                        (2 CT-es)                      (5 CT-es)

        [cte_g01]                                                        [cte_g10] ────┐
            │                                                               ▲          │
            ▼                                                               │          ▼
        [cte_g02] ──┬──> [cte_g03] ──┐                                  [cte_g08]   [cte_g12]
                    │                ▼                                      ▲          ▲
                    └──> [cte_g04] ──┴──> [cte_g05] ──> [cte_g06] ──>   [cte_g07]      │
                                                                            │          │
                                                                            ▼          │
                                                                         [cte_g09] ────┤
                                                                            │          │
                                                                            ▼          │
                                                                         [cte_g11] ────┘

        */

        println!("grafo_original: {grafo_original:#?}");

        // ---------------------------------------------------------------------
        // EXECUÇÃO DOS DOIS MÉTODOS SOBRE CÓPIAS INDEPENDENTES
        // ---------------------------------------------------------------------
        let mut grafo_dsu = grafo_original.clone();
        let mut grafo_old = grafo_original.clone();

        grafo_dsu.expand_ctes(false);
        grafo_old.expand_ctes_old(false);

        println!("grafo_dsu: {grafo_dsu:#?}");

        // ---------------------------------------------------------------------
        // VALIDAÇÃO: OS RESULTADOS DEVEM SER 100% IGUAIS
        // ---------------------------------------------------------------------
        assert_eq!(
            grafo_dsu, grafo_old,
            "Inconsistência detectada: o resultado do DSU difere do algoritmo de Ponto Fixo!"
        );

        // ---------------------------------------------------------------------
        // VALIDAÇÕES ADICIONAIS DE INTEGRIDADE CONTÁBIL
        // ---------------------------------------------------------------------
        // Total de nós ativos participantes: 2 + 3 + 5 + 8 + 12 = 30 documentos
        assert_eq!(grafo_dsu.len(), 30);

        // Grupo 1: Cada membro do par deve ter exatamente 1 vizinho
        assert_eq!(grafo_dsu.get("cte_p1").unwrap().len(), 1);
        assert!(grafo_dsu.get("cte_p1").unwrap().contains("cte_p2"));

        // Grupo 2: Cada membro do trio deve ter exatamente 2 vizinhos
        assert_eq!(grafo_dsu.get("cte_t1").unwrap().len(), 2);
        assert!(grafo_dsu.get("cte_t1").unwrap().contains("cte_t3"));

        // Grupo 3: Cada membro do quinteto deve ter exatamente 4 vizinhos
        assert_eq!(grafo_dsu.get("cte_q1").unwrap().len(), 4);
        assert!(grafo_dsu.get("cte_q4").unwrap().contains("cte_q1"));

        // Grupo 4: Cada membro da estrela (8 nós) deve ter exatamente 7 vizinhos
        assert_eq!(grafo_dsu.get("cte_est_hub").unwrap().len(), 7);
        assert_eq!(grafo_dsu.get("cte_est_01").unwrap().len(), 7);

        // Grupo 5: Cada membro da rede de 12 nós deve ter exatamente 11 vizinhos
        assert_eq!(grafo_dsu.get("cte_g01").unwrap().len(), 11);
        assert_eq!(grafo_dsu.get("cte_g12").unwrap().len(), 11);
        assert!(grafo_dsu.get("cte_g01").unwrap().contains("cte_g12"));

        // Anti-reflexividade: nenhum nó pode conter a si mesmo
        for (no, vizinhos) in &grafo_dsu {
            assert!(
                !vizinhos.contains(no),
                "O nó '{}' contém a si próprio no conjunto de vizinhos!",
                no
            );
        }
    }
}

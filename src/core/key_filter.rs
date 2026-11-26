//! # Filtro de Documentos Fiscais por Chaves de Acesso (`-f` / `--filtro`)
//!
//! Retém apenas os documentos cujas chaves de acesso de 44 caracteres constam em um
//! arquivo texto (uma chave por linha; linhas em branco e comentários `#` são ignorados).
//!
//! O filtro é aplicado logo após a desserialização de cada XML — antes de amostragem,
//! correlação e exportação — tanto no motor em memória quanto no out-of-core (`-b`),
//! de modo que documentos descartados nunca chegam a ser gravados nos shards.
//!
//! Semântica:
//! - NF-e/NFC-e: itens retidos quando a chave do documento está no arquivo.
//! - CT-e (e variantes GTV-e/Simplificado/OS): retido quando a própria chave está no
//!   arquivo ou quando alguma NF-e referenciada está no arquivo.
//! - Eventos e cancelamentos: retidos quando a chave do documento afetado está no arquivo.
//! - e-Financeira: descartada enquanto o filtro estiver ativo (não possui chave de 44 caracteres).
//!
//! Com `-e`, as chaves ausentes vão para stdout (uma por linha, sem cabeçalho),
//! permitindo `> chaves_nao_encontradas.txt`; o balanço final permanece em stderr.

use std::{
    collections::HashSet,
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
};

use crate::{Arguments, Information, XmlParserError, XmlParserResult};

/// Filtro global inicializado uma única vez a partir de `--filtro`.
static KEY_FILTER: OnceLock<KeyFilter> = OnceLock::new();

/// Conjunto de chaves de acesso de 44 caracteres usado para filtrar documentos fiscais.
#[derive(Debug)]
pub struct KeyFilter {
    path: PathBuf,
    keys: HashSet<String>,
    found: Mutex<HashSet<String>>,
}

impl KeyFilter {
    /// Lê o arquivo texto de chaves (uma por linha), validando e deduplicando as entradas.
    pub fn load(path: &Path) -> XmlParserResult<Self> {
        let file = File::open(path).map_err(|source| XmlParserError::IoContext {
            source,
            path: path.to_path_buf(),
        })?;
        let reader = BufReader::new(file);
        let mut keys = HashSet::new();
        let mut invalid_count = 0usize;
        let mut invalid_sample: Vec<String> = Vec::new();

        for (idx, line) in reader.lines().enumerate() {
            let line = line.map_err(|source| XmlParserError::IoContext {
                source,
                path: path.to_path_buf(),
            })?;
            // Remove BOM UTF-8 eventual, espaços e normaliza via crate::normalize_key (caixa alta).
            let Some(key) = crate::normalize_key(line.trim_start_matches('\u{feff}')) else {
                continue;
            };
            if key.starts_with('#') {
                continue;
            }
            if key.len() == 44 && key.chars().all(|c| c.is_ascii_alphanumeric()) {
                keys.insert(key);
            } else {
                invalid_count += 1;
                if invalid_sample.len() < 5 {
                    invalid_sample.push(format!("linha {}: {key:?}", idx + 1));
                }
            }
        }

        if invalid_count > 0 {
            return Err(XmlParserError::InvalidDocument(format!(
                "Arquivo de chaves '{}' contém {invalid_count} entrada(s) inválida(s) \
                 (esperado: 44 caracteres [0-9A-Z]): {}",
                path.display(),
                invalid_sample.join("; ")
            )));
        }
        if keys.is_empty() {
            return Err(XmlParserError::InvalidDocument(format!(
                "Arquivo de chaves '{}' não contém nenhuma chave válida de 44 caracteres.",
                path.display()
            )));
        }

        Ok(Self {
            path: path.to_path_buf(),
            keys,
            found: Mutex::new(HashSet::new()),
        })
    }

    /// Retorna `true` se a chave informada consta no arquivo de filtro (case-insensitive).
    #[inline]
    pub fn contains(&self, chave: &str) -> bool {
        if chave.bytes().any(|b| b.is_ascii_lowercase()) {
            crate::normalize_key(chave).is_some_and(|k| self.keys.contains(&k))
        } else {
            self.keys.contains(chave)
        }
    }

    /// Quantidade de chaves únicas solicitadas.
    pub fn len(&self) -> usize {
        self.keys.len()
    }

    /// Retorna `true` se o filtro não contém nenhuma chave.
    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    /// Caminho do arquivo texto de origem das chaves.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Registra a chave de um documento efetivamente localizado no lote.
    fn mark_found(&self, chave: &str) {
        let Some(norm) = crate::normalize_key(chave) else {
            return;
        };
        if self.keys.contains(&norm)
            && let Ok(mut found) = self.found.lock()
        {
            found.insert(norm);
        }
    }

    /// Quantidade de chaves solicitadas localizadas em documentos do lote.
    pub fn found_count(&self) -> usize {
        self.found.lock().map(|g| g.len()).unwrap_or(0)
    }

    /// Chaves solicitadas que não foram localizadas em nenhum documento do lote (ordenadas).
    pub fn missing_keys(&self) -> Vec<String> {
        let found = self.found.lock().map(|g| g.clone()).unwrap_or_default();
        let mut missing: Vec<String> = self.keys.difference(&found).cloned().collect();
        missing.sort();
        missing
    }

    /// Aplica o filtro a um documento recém-desserializado, retendo apenas o que casa.
    pub fn retain(&self, info: Information) -> Option<Information> {
        match info {
            Information::Cte(c) => {
                let own = c.cte.as_deref().is_some_and(|k| self.contains(k));
                let linked = c
                    .nfes_vinculados
                    .iter()
                    .chain(c.nfes.iter())
                    .any(|k| self.contains(k));
                if !own && !linked {
                    return None;
                }
                if let Some(k) = c.cte.as_deref() {
                    self.mark_found(k);
                }
                Some(Information::Cte(c))
            }
            Information::Nfe(items) => {
                let kept: Vec<_> = items
                    .into_iter()
                    .filter(|n| n.nfe.as_deref().is_some_and(|k| self.contains(k)))
                    .collect();
                let first = kept.first()?.nfe.clone()?;
                self.mark_found(&first);
                Some(Information::Nfe(kept))
            }
            Information::EventoCte(e) => {
                let keep = e.cte.as_deref().is_some_and(|k| self.contains(k));
                keep.then_some(Information::EventoCte(e))
            }
            Information::EventoNfe(e) => {
                let keep = e.nfe.as_deref().is_some_and(|k| self.contains(k));
                keep.then_some(Information::EventoNfe(e))
            }
            Information::CancelamentoCte(c) => {
                let keep = c.cte.as_deref().is_some_and(|k| self.contains(k));
                keep.then_some(Information::CancelamentoCte(c))
            }
            Information::CancelamentoNfe(c) => {
                let keep = c.nfe.as_deref().is_some_and(|k| self.contains(k));
                keep.then_some(Information::CancelamentoNfe(c))
            }
            // e-Financeira não possui chave de acesso de 44 caracteres.
            Information::EFinanceira(_) | Information::None => None,
        }
    }
}

/// Inicializa o filtro global a partir de `--filtro` (chamado uma vez no início da execução).
pub fn init_key_filter(arguments: &Arguments) -> XmlParserResult<()> {
    if let Some(path) = &arguments.filtro {
        let filter = KeyFilter::load(path)?;
        eprintln!(
            "Filtro de chaves: {} chave(s) única(s) carregada(s) de '{}'",
            filter.len(),
            filter.path().display()
        );
        let _ = KEY_FILTER.set(filter);
    }
    Ok(())
}

/// Acesso ao filtro global ativo (se `--filtro` foi informado).
pub fn active_key_filter() -> Option<&'static KeyFilter> {
    KEY_FILTER.get()
}

/// Aplica o filtro global (quando ativo) a um documento recém-desserializado.
///
/// Sem filtro ativo, é um passthrough de custo desprezível (uma leitura de `OnceLock`).
pub fn filter_information(info: Information) -> Option<Information> {
    match active_key_filter() {
        Some(filter) => filter.retain(info),
        None => Some(info),
    }
}

/// Exibe ao final da execução o balanço das chaves solicitadas versus encontradas.
///
/// O balanço (solicitadas | encontradas | não encontradas) é sempre emitido em stderr.
///
/// Com `-e`, as chaves ausentes são emitidas em stdout, uma por linha e sem cabeçalho,
/// permitindo redirecionamento direto para arquivo reutilizável como `--filtro`:
///
/// read-xml -catr -f chaves_extraidas.txt -e > chaves_nao_encontradas.txt
pub fn print_key_filter_report(arguments: &Arguments) {
    let Some(filter) = active_key_filter() else {
        return;
    };
    let missing = filter.missing_keys();
    eprintln!(
        "Filtro de chaves: {} solicitada(s) | {} encontrada(s) | {} não encontrada(s)",
        filter.len(),
        filter.found_count(),
        missing.len()
    );
    // stdout carrega apenas dados (chaves puras); stderr carrega o diagnóstico acima.
    // Escrita única em buffer: o stdout é line-buffered (1 syscall por linha);
    // montar um buffer e imprimir uma vez evita ~N syscalls e garante exatamente
    // N linhas, sem linha em branco residual.
    if arguments.exibir_chaves_nao_encontradas && !missing.is_empty() {
        let mut buf = String::with_capacity(missing.len() * 45);
        for chave in &missing {
            buf.push_str(chave);
            buf.push('\n');
        }
        print!("{buf}");
    }
}

//----------------------------------------------------------------------------//
//                                   Tests                                    //
//----------------------------------------------------------------------------//

/// Run tests with:
/// cargo test -- --show-output tests_key_filter
#[cfg(test)]
mod tests_key_filter {
    use super::*;
    use crate::xml_structs::{cte::InfoCte, cte_evento::InfoCteEvento, nfe::InfoNfe};

    const CHAVE_A: &str = "35240147960950000121550020005910991037128568";
    const CHAVE_B: &str = "35240147960950000121550020005911001035434869";
    const CHAVE_C: &str = "35240147960950000121550020005911011063932557";
    const CHAVE_CTE: &str = "35240147960950000121570020000001231000000015";

    fn filter_with(keys: &[&str]) -> KeyFilter {
        KeyFilter {
            path: PathBuf::from("teste.txt"),
            keys: keys
                .iter()
                .filter_map(|k| crate::normalize_key(k))
                .collect(),
            found: Mutex::new(HashSet::new()),
        }
    }

    #[test]
    fn retain_aceita_chave_alfanumerica_case_insensitive() -> XmlParserResult<()> {
        let alfa_upper = "35260112ABC3450001DE550010000001231000000123";
        assert_eq!(alfa_upper.len(), 44);
        let path =
            std::env::temp_dir().join(format!("read_xml_keys_alfa_{}.txt", std::process::id()));
        std::fs::write(&path, format!("{alfa_upper}\n"))?;
        let filter = KeyFilter::load(&path)?;
        let _ = std::fs::remove_file(&path);

        let nfe = InfoNfe {
            nfe: Some(alfa_upper.to_ascii_lowercase()),
            ..Default::default()
        };
        assert!(filter.retain(Information::Nfe(vec![nfe])).is_some());
        assert_eq!(filter.found_count(), 1);
        assert!(filter.missing_keys().is_empty());
        Ok(())
    }

    #[test]
    fn load_valida_deduplica_ignora_comentarios() -> XmlParserResult<()> {
        assert_eq!(CHAVE_A.len(), 44);
        assert_eq!(CHAVE_B.len(), 44);
        let path = std::env::temp_dir().join(format!("read_xml_keys_{}.txt", std::process::id()));
        std::fs::write(
            &path,
            format!("# comentario\n\n{CHAVE_A}\n  {CHAVE_A}  \n{CHAVE_B}\n"),
        )?;
        let filter = KeyFilter::load(&path)?;
        let _ = std::fs::remove_file(&path);
        assert_eq!(filter.len(), 2);
        assert!(filter.contains(CHAVE_A));
        assert!(!filter.contains(CHAVE_CTE));
        Ok(())
    }

    #[test]
    fn load_rejeita_chave_invalida() -> XmlParserResult<()> {
        let path =
            std::env::temp_dir().join(format!("read_xml_keys_bad_{}.txt", std::process::id()));
        std::fs::write(&path, "123\n")?;
        let res = KeyFilter::load(&path);
        let _ = std::fs::remove_file(&path);
        assert!(matches!(res, Err(XmlParserError::InvalidDocument(_))));
        Ok(())
    }

    #[test]
    fn retain_nfe_por_chave_e_reporta_ausencia() {
        let filter = filter_with(&[CHAVE_A, CHAVE_B]);
        // Dois itens da mesma NF-e (chave A) e um item de outra NF-e fora do filtro (chave C).
        let itens = vec![
            InfoNfe {
                nfe: Some(CHAVE_A.to_string()),
                ..Default::default()
            },
            InfoNfe {
                nfe: Some(CHAVE_A.to_string()),
                ..Default::default()
            },
            InfoNfe {
                nfe: Some(CHAVE_C.to_string()),
                ..Default::default()
            },
        ];
        match filter.retain(Information::Nfe(itens)) {
            Some(Information::Nfe(v)) => {
                assert_eq!(v.len(), 2);
                assert!(v.iter().all(|n| n.nfe.as_deref() == Some(CHAVE_A)));
            }
            other => panic!("esperado Nfe filtrada, obtido {other:?}"),
        }
        // Dois itens com a mesma chave contam como uma única chave encontrada.
        assert_eq!(filter.found_count(), 1);
        // CHAVE_B foi solicitada mas não apareceu em nenhum documento do lote.
        assert_eq!(filter.missing_keys(), vec![CHAVE_B.to_string()]);
    }

    #[test]
    fn retain_cte_proprio_ou_vinculado() {
        let filter = filter_with(&[CHAVE_A]);
        let cte_vinculado = InfoCte {
            cte: Some(CHAVE_CTE.to_string()),
            nfes_vinculados: vec![CHAVE_A.to_string()],
            ..Default::default()
        };
        assert!(
            filter
                .retain(Information::Cte(Box::new(cte_vinculado)))
                .is_some()
        );
        // A chave própria do CTe não está no filtro: não deve ser marcada como encontrada.
        assert_eq!(filter.found_count(), 0);

        let cte_sem_relacao = InfoCte {
            cte: Some(CHAVE_CTE.to_string()),
            nfes_vinculados: vec![CHAVE_B.to_string()],
            ..Default::default()
        };
        assert!(
            filter
                .retain(Information::Cte(Box::new(cte_sem_relacao)))
                .is_none()
        );
    }

    #[test]
    fn retain_eventos_e_descarta_efinanceira() {
        let filter = filter_with(&[CHAVE_A]);
        let ev = InfoCteEvento {
            cte: Some(CHAVE_A.to_string()),
            ..Default::default()
        };
        assert!(
            filter
                .retain(Information::EventoCte(Box::new(ev)))
                .is_some()
        );
        let ev_fora = InfoCteEvento {
            cte: Some(CHAVE_B.to_string()),
            ..Default::default()
        };
        assert!(
            filter
                .retain(Information::EventoCte(Box::new(ev_fora)))
                .is_none()
        );
        assert!(
            filter
                .retain(Information::EFinanceira(Vec::new()))
                .is_none()
        );
        assert!(filter.retain(Information::None).is_none());
    }
}

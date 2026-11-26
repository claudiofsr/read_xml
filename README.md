# `read-xml`

Motor de alta performance em Rust para leitura recursiva em streaming de documentos fiscais eletrônicos brasileiros (**NF-e**, **NFC-e**, **CT-e**, **GTV-e**, **CT-e Simplificado**, **CT-e OS**, **Eventos/Cancelamentos** e **e-Financeira**) diretamente dentro de arquivos `.zip` (inclusive `.zip` aninhados) e de arquivos `.xml` já descompactados, com correlação transitiva via grafo (`DSU` / *Union-Find*), exportação para **XLSX** / **CSV** (incluindo o layout de 56 colunas compatível com a Receita Federal), amostragem estratificada em streaming, auditoria de schemas (`-k`) e modo *out-of-core* (`-b`).

---

## Princípios de Fidelidade ao XML

Os arquivos `.xml` autorizados são tratados como a **única fonte da verdade**:

1. **Reter informações existentes no XML**: extrair integralmente valores, tributos, despesas acessórias (`vOutro`), `det/vItem`, `ICMS60` (CT-e), `ICMS61` (monofásico) e metadados fiscais conforme o documento original.
2. **Não omitir dados existentes no XML**: preservar dados de remetente, destinatário, tomador, percurso, observações (`xObs` / `infAdic`) e chaves correlacionadas (CT-es e NF-es).
3. **Não sobrescrever campos cadastrais**: preservar nomes (`xNome`) e municípios (`xMun`) originais do XML, aplicando votação canônica apenas como *fallback* quando ausentes (`<N/D>`).
4. **Não aplicar inferências divergentes do XML**: manter estrita a Tabela 4.3.7 da EFD-Contribuições (Natureza da Base de Cálculo somente para CSTs geradores de crédito `50..=56` e `60..=66`) e calcular o valor total do item pela prioridade oficial **`det/vItem` -> fórmula VB01-10 / W16-10** (NT 2023.004 v1.20, NT 2025.002-RTC e NT 2026.008 v1.01), recorrendo a rateio proporcional apenas se o item não possuir componentes suficientes.

---

## Base Legal e Cálculo do Valor do Item / Valor Proporcional

Em conformidade com a legislação fiscal brasileira vigente em novembro de 2026:

- **Lei Complementar nº 87/1996 (Lei Kandir, art. 13, § 1º)** — integra frete, seguro e demais despesas acessórias cobradas do destinatário na base da operação: https://www.planalto.gov.br/ccivil_03/leis/lcp/lcp87.htm
- **Lei nº 10.833/2003 (art. 1º, § 3º, V, "a")** — exclusão dos descontos incondicionais concedidos na nota fiscal: https://www.planalto.gov.br/ccivil_03/leis/2003/l10.833.htm
- **Decreto nº 7.212/2010 (RIPI, art. 190, § 1º)** — o valor da operação compreende o preço do produto acrescido do frete e das demais despesas acessórias cobradas ou debitadas ao comprador: https://www.planalto.gov.br/ccivil_03/_ato2007-2010/2010/decreto/d7212.htm
- **Ajuste SINIEF nº 07/2005 (CONFAZ)** — institui a NF-e e vincula o leiaute e as regras de validação ao Manual de Orientação do Contribuinte (MOC) e às Notas Técnicas publicadas no Portal Nacional: https://www.confaz.fazenda.gov.br/legislacao/ajustes/2005/aj_007_05
- **Lei Complementar nº 214/2025 (Planalto)** — institui o IBS, a CBS e o Imposto Seletivo (IS): https://www.planalto.gov.br/ccivil_03/leis/lcp/lcp214.htm
- **Decreto nº 12.955/2026 (Planalto)** — regulamenta a CBS: https://www.planalto.gov.br/ccivil_03/_ato2023-2026/2026/decreto/d12955.htm
- **Ato Conjunto RFB/CGIBS nº 1/2025 (DOU)** — obrigações acessórias do IBS/CBS em 2026: https://www.in.gov.br/web/dou/-/ato-conjunto-rfb/cgibs-n-1-de-22-de-dezembro-de-2025-677624586
- **Portal Nacional da NF-e — Notas Técnicas vigentes** (**NT 2023.004 v1.20** / Regra `W16-10`, **NT 2025.002-RTC v1.53** / `<det><vItem>` e Regra `VB01-10`, **NT 2026.008 v1.01**): https://www.nfe.fazenda.gov.br/portal/listaConteudo.aspx?tipoConteudo=04BIflQt1aY=
- **Portal Nacional da NF-e — Manuais (MOC 7.0)**: https://www.nfe.fazenda.gov.br/portal/listaConteudo.aspx?tipoConteudo=ndIjl+iEFdE=
- **Portal do CT-e — Notas Técnicas**: https://www.cte.fazenda.gov.br/portal/listaConteudo.aspx?tipoConteudo=Y0nErnoZpsg=

### Prioridade de Cálculo por Item (`Valor Total do Item (VB01-10)` e `Valor da Nota Proporcional`)

1. **`<det><vItem>` (NT 2025.002-RTC / NT 2026.008 v1.01)**: utilizado diretamente quando informado no XML.
2. **Fórmula oficial `VB01-10` / `W16-10` (NT 2023.004 v1.20)**:

```text
vItem = vProd - vDesc - vICMSDeson (se indDeduzDeson = 1)
      + vICMSST + vICMSMonoReten + vFCPST
      + vFrete + vSeg + vOutro + vII + vIPI + vIPIDevol
      + PISST (se indSomaPISST = 1) + COFINSST (se indSomaCOFINSST = 1)
```

*(Nos anos de 2025 e 2026, `IBS`, `CBS` e `IS` não integram `vProd` nem somam em `vItem`/`vNF`, conforme Exceção 1 das NTs vigentes.)*

3. **Rateio proporcional a `vNF` (último recurso)**: aplicado apenas quando a soma dos itens diferir de `vNF` em mais de R$ 0,02 (por exemplo, acréscimos/descontos globais não discriminados em `<det>`).

---

## Principais Recursos

- **Leitura direta de `.zip` e `.xml` soltos (sem extração em disco)**: processa 1 `.zip` por vez com paralelismo intra-ZIP via `pread` + `rayon`, mantendo o consumo de RAM reduzido mesmo em lotes com dezenas de milhões de XMLs.
- **Dois modos de execução**:
  - **Em memória (padrão)**: máxima velocidade em RAM.
  - **Out-of-core (`-b` / `--buffer`)**: particionamento em disco com *hash sharding* (`postcard` + `zstd`), grafo compacto `CteComponents` $O(V)$ e *K-Way External Merge Sort*.
- **Amostragem estratificada em streaming (`--amostragem`)**:
  - Memória $O(n)$ por tipo de documento fiscal (CT-e, NF-e completa com todos os seus itens e e-Financeira).
  - Combina **maiores valores** (`--maiores x`) e **amostragem aleatória uniforme disjunta** (`--random y`), com `--seed` opcional para reprodutibilidade.
- **Compatibilidade com leiautes oficiais vigentes (novembro de 2026)**:
  - **NF-e / NFC-e**: v4.00, NT 2023.004 (`W16-10`, `indDeduzDeson`), NT 2024.003 (`agropecuario`), NT 2025.002 (`IBSCBS`, `IS`, `det/vItem`, `VB01-10`, monofásico Ad Rem / Ad Valorem, `gALCZFMCBS`, `cIndOp`, `DFeReferenciado`), NT 2026.006 (`gPgtoVinc`) e NT 2026.008 v1.01 (`vProdLiq`, `vProdLiqTot`, `ICMSPrevistoPagtoAntecip`).
  - **CT-e**: v3.00 / v4.00 (`cteProc`, `CTe`), GTV-e mod. 64 (`GTVeProc`/`GTVe`), CT-e Simplificado (`cteSimpProc`/`CTeSimp`), CT-e OS mod. 67 (`cteOSProc`/`CTeOS`), NT 2026.001 (`pgtoVinc`), NT 2026.002 (`pagAnt`, `tpPagAnt`, `ISUFemit`), NT 2026.004 (`gMultaJuros/chCTeRef`, CNPJ alfanumérico) e `<dest><locEnt>` / `<rem><locColeta>`.
  - **Eventos e Cancelamentos**: `procEventoCTe`, `eventoCTe`, `procEventoNFe`, `evento`, `procCancCTe`, `procCancNFe`, incluindo homologação por `cStat` `101`, `135`, `136`, `151` e `155`, registro de passagem (`modalOutro`, `ctg`), comprovante de entrega (`evCECTe`) e pedidos de prorrogação sem falsos positivos.
  - **e-Financeira (SPED)**: `evtMovOpFin`, `evtMovOpFinAnual`, `loteEventos`, `loteEventosAssincrono` (v1.2.1 a v1.6.0 / ADE Cofis nº 17/2026).
- **Tratamento robusto de encodings e caracteres**:
  - Detecção automática de BOM (`UTF-8`, `UTF-16LE`, `UTF-16BE`) e declaração XML (`UTF-8`, `ISO-8859-1`, `Windows-1252`), com *fallback* automático para `Windows-1252`.
  - Sanitização de `\r\n`, `\r`, `\n` e `\t`, além de `--clean-spaces` para colapsar espaços múltiplos.
  - Normalização canônica em **caixa alta (`A-Z0-9`)** para chaves de acesso de 44 caracteres e suporte completo a **CNPJ alfanumérico** (`format_cnpj_alnum`).
- **Correlação completa via Grafo `DSU` (`CteComponents` $O(V)$)**:
  - Calcula componentes conexas de CT-es e propaga correlações bidirecionais (`CT-e <-> NF-e`, `CT-e <-> CT-e`, `NF-e <-> CT-e`, `NF-e <-> NF-e`).
- **Layout de 56 colunas (`-r` / `--receita`)**:
  - Por padrão, exibe **todas** as chaves correlacionadas (CT-es e NF-es) na coluna `Inf. NFe - Chave de acesso da NF-e : ConhecimentoInformacaoNFe`, tanto em linhas de CT-e quanto de NF-e.
  - Use `--diversos` para reproduzir estritamente os extratos do ContÁgil/Receita Federal (`*DIVERSOS*` em CT-es com múltiplas NF-es e `<N/D>` em NF-es).
- **Filtro por chaves de acesso (`-f` / `--filtro <ARQUIVO>`)**:
  - Retém em streaming apenas documentos cujas chaves de 44 caracteres constam no arquivo texto; com `-e`, lista as chaves ausentes em `stdout` (uma por linha, sem cabeçalho) e o balanço em `stderr`.
- **Auditoria de consistência (`-k` / `--verificar-consistencia`)**:
  - Varredura SAX concorrente em streaming ($O(1)$ por XML validado) diretamente nos `.zip`s e `.xml`s soltos para identificar tags não mapeadas.

---

## Instalação

Pré-requisito: Rust toolchain (`rust-version = "1.100"`, edição 2024).

```bash
git clone https://github.com/claudiofsr/read_xml.git
cd read_xml
cargo build --release
cargo install --path=.
```

---

## Uso e Opções de Linha de Comando

```text
read-xml [OPTIONS]
```

| Opção | Descrição |
| --- | --- |
| `--amostragem <AMOSTRAGEM>` | Retém no máximo N documentos fiscais por tipo durante a extração XML (amostragem em streaming) |
| `--maiores <AMOSTRA_MAIORES>` | Com `--amostragem`: retém X documentos fiscais com os maiores valores |
| `--random <AMOSTRA_ALEATORIA>` | Com `--amostragem`: retém Y documentos fiscais aleatórios uniformemente |
| `--seed <SAMPLE_SEED>` | Com `--amostragem`: semente determinística opcional para reprodutibilidade |
| `-p, --path <PATH>` | Caminho de arquivo `.zip`/`.xml` ou diretório (padrão: busca recursiva no diretório atual) |
| `-m, --max_depth <MAX_DEPTH>` | Profundidade máxima na busca recursiva de arquivos |
| `-c, --csv` | Gera arquivos CSV (`;` por padrão) |
| `-d, --delimiter <DELIMITER>` | Define o delimitador CSV (ou via `DELIMITER_CSV`; padrão: `;`) |
| `-a, --avoid` | Evita gerar planilhas `.xlsx` |
| `--memory-mode <MODE>` | Modo de memória do Excel: `constant`, `low` (padrão) ou `memory` |
| `-b, --buffer` | Execução *out-of-core* em disco (limita pico de RAM e força Excel `constant`) |
| `-r, --receita` | Converte saída de CT-e e NF-e para o layout de 56 colunas |
| `-f, --filtro <ARQUIVO>` | Filtra documentos fiscais pelas chaves de 44 caracteres listadas em arquivo texto (alias: `--chaves`) |
| `--format-br` | Com `-r`: formata floats no padrão brasileiro (`1.234,56`) em vez do padrão nativo (`1234.56`) |
| `--diversos` | Com `-r`: usa `*DIVERSOS*` em CT-es e `<N/D>` em NF-es (compatível com extratos da Receita/ContÁgil) |
| `--clean-spaces` | Colapsa dois ou mais espaços consecutivos em um único espaço |
| `-i, --itens <ITENS>` | Quantidade máxima de itens NCM/Tomador consolidados (padrão: `100`) |
| `-l, --linhas <LINHAS>` | Gera arquivos `.txt` particionados (`nfes-00001.txt`, `ctes-00001.txt`) com até N chaves (`>= 900`) |
| `-e, --exibir-chaves-nao-encontradas` | Exibe chaves referenciadas não encontradas; com `--filtro`, lista chaves ausentes do filtro em `stdout` |
| `--exibir-correlacoes` | Exibe o mapa detalhado de correlações CT-e ↔ NF-e |
| `-k, --verificar-consistencia` | Verifica tags XML não mapeadas nos `.zip`s e `.xml`s soltos (sem gerar planilhas) |
| `-s, --structure <XML>` | Infere e imprime struct Rust compatível com `serde` a partir de um XML |
| `-n, --nodes <XML>` | Lista todos os caminhos e nós de um arquivo XML |
| `-g, --generate <SHELL>` | Gera autocompletar para shell (`bash`, `elvish`, `fish`, `powershell`, `zsh`) |
| `-t, --time` | Exibe o tempo total de execução |
| `-w, --wipe_terminal` | Limpa a tela do terminal antes da execução |
| `-v, --verbose` | Exibe mensagens detalhadas durante o processamento |

### Exemplos Práticos

```bash
# Processar todos os .zip e .xml em modo out-of-core gerando CSVs no layout de 56 colunas
# exibindo todas as chaves correlacionadas (CT-es e NF-es)
read-xml -catrb --clean-spaces

# Reproduzir o formato numérico brasileiro e o padrão estrito da Receita/ContÁgil (*DIVERSOS* / <N/D>)
read-xml -catrb --format-br --diversos --clean-spaces

# Filtrar documentos pelas chaves de 44 caracteres e salvar chaves ausentes em arquivo
read-xml -catrb -f chaves_extraidas.txt -e > chaves_nao_encontradas.txt

# Amostra de 1.000 documentos por tipo (800 maiores valores + 200 aleatórios com seed fixa)
read-xml -catrb --amostragem 1000 --maiores 800 --random 200 --seed 1973

# Auditar tags XML não mapeadas diretamente nos arquivos .zip (29M+ XMLs com < 1 GB de RAM)
read-xml -k

# Gerar struct Rust a partir de um XML de exemplo
read-xml -s documento.xml > struct_gerada.rs
```

---

## Arquitetura do Projeto

```text
read_xml/
├── scripts/
│   ├── atualizar_cfop.py       # Atualiza src/export/cfop.rs via CONFAZ / XLSX / CSV
│   ├── atualizar_ncm.py        # Atualiza src/export/ncm.rs via API Siscomex/RFB
│   ├── auditar-valor-proporcional.py # Audita XML original vs ContÁgil vs read-xml
│   ├── comparar-receita.py     # Compara CSVs gerados contra extratos da Receita Federal
│   └── gerar_validacao.py      # Sincroniza src/xml_validation.rs e audita ZIPs reais
└── src/
    ├── main.rs                 # Ponto de entrada e seleção do pipeline
    ├── lib.rs                  # Fachada pública da biblioteca
    ├── error.rs                # Definição tipada de erros (XmlParserError)
    ├── xml_validation.rs        # Varredura SAX e validação de tags via serde_introspect
    ├── cli/                    # Argumentos (clap), barras de progresso e inspeção de nós
    ├── core/                   # Domínio fiscal, agregações, eventos, grafo DSU e amostragem
    ├── pipeline/               # Motores in-memory e out-of-core (zstd + postcard + k-way merge)
    ├── export/                 # Exportadores CSV/XLSX, layout 56 colunas e tabelas (cfop, cst, natureza, ncm)
    ├── utils/                  # Leitura de encodings, sanitização de texto, normalização e regex
    └── xml_structs/            # Schemas Serde (NF-e, CT-e, GTV-e, CTeSimp, CTeOS, Eventos, e-Financeira)
```

---

## Manutenção das Tabelas Oficiais e Testes

```bash
# Atualizar tabelas oficiais (opcional)
./scripts/atualizar_cfop.py
./scripts/atualizar_ncm.py

# Verificar sincronismo entre structs XML e validador (-k)
./scripts/gerar_validacao.py --check

# Formatação, lint, testes e benchmarks
cargo fmt --check
cargo clippy --all-targets
cargo test
cargo bench
```

---

## Licença

Distribuído sob a licença **BSD-3-Clause**. Consulte os arquivos de licença no repositório para mais detalhes.

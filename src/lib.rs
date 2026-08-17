//! # Motor de Processamento e Análise de Documentos Fiscais Eletrônicos (DF-e)
//!
//! O `read_xml` realiza a varredura concorrente, correlação transitiva,
//! validação estrutural e exportação de documentos fiscais brasileiros (NF-e, CT-e, e-Financeira).

/*
src/
├── main.rs                  # Ponto de entrada (CLI Runner / Error Boundary)
├── lib.rs                   # Fachada pública e orquestração dos submódulos
├── error.rs                 # Definição global de erros (XmlParserError)
│
├── cli/                     # INTERFACE DE LINHA DE COMANDO E TERMINAL
│   ├── mod.rs
│   ├── args.rs              # Definição dos argumentos (Clap)
│   ├── progress.rs          # Gerenciamento de barras de progresso (Indicatif)
│   └── nodes.rs             # Inspeção de nós XML de baixo nível
│
├── core/                    # MOTOR DE REGRAS DE NEGÓCIO E DOMÍNIO FISCAL
│   ├── mod.rs
│   ├── docs_fiscais.rs      # Repositório consolidado (DocsFiscais)
│   ├── information.rs       # Despachante de parsers (Information / StructExtension)
│   ├── event.rs             # Vinculação de eventos e cancelamentos
│   ├── graph.rs             # Algoritmo DSU e fecho transitivo CT-e <-> NF-e
│   └── aggregations.rs      # Agrupamentos analíticos por NCM/Tomador
│
├── export/                  # GERADORES DE SAÍDA TABULAR
│   ├── mod.rs
│   ├── excel.rs             # Serialização e streaming para XLSX (rust_xlsxwriter)
│   ├── format.rs            # Estilos, fontes e regras visuais de células
│   ├── memory_mode.rs       # Perfis de consumo de RAM (Constant, Low, Memory)
│   └── csv.rs               # Gravador otimizado de CSV
│
├── validation/              # AUDITORIA E INTEGRIDADE DE SCHEMAS
│   ├── mod.rs
│   └── xml_validation.rs    # Varredura SAX para detecção de tags não mapeadas
│
├── utils/                   # UTILITÁRIOS, TRAITS AUXILIARES E REGEX
│   ├── mod.rs
│   ├── common.rs            # Manipulação de caminhos, datas NaiveDate e IO
│   ├── regex.rs             # Compilação estática de Regex e RegexSet
│   ├── traits.rs            # Extensões para Option, String e Date
│   ├── group_by_hashmap.rs  # Extensão para agrupamento concorrente via Rayon
│   └── unique_with_cows.rs  # Deduplicação de coleções com Copy-on-Write
│
└── xml_structs/             # SCHEMAS XML (SERDE DTOs)
    ├── mod.rs
    ├── agente.rs
    ├── assinaturas.rs
    ├── aut_xml.rs
    ├── cancelamento.rs
    ├── cancelamento_cte.rs
    ├── cancelamento_nfe.rs
    ├── cobranca.rs
    ├── cte.rs
    ├── cte_detalhamento.rs
    ├── cte_evento.rs
    ├── efinanceira.rs
    ├── endereco.rs
    ├── entrega.rs
    ├── impostos.rs
    ├── integrated_dev_env.rs
    ├── nfe.rs
    ├── nfe_detalhamento.rs
    ├── nfe_evento.rs
    ├── pagamento.rs
    └── ret_evento.rs
*/

mod cli;
mod core;
mod error;
mod example01;
mod export;
mod utils;
mod xml_structs;
mod xml_validation;

pub use self::{
    cli::*,
    core::*,
    error::{XmlParserError, XmlParserResult},
    export::*,
    utils::*,
    xml_structs::*,
    xml_validation::*,
};

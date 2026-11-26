//! # Motor de Processamento e Análise de Documentos Fiscais Eletrônicos (DF-e)
//!
//! O `read_xml` realiza a varredura concorrente, correlação transitiva,
//! validação estrutural, amostragem estratificada e exportação de documentos fiscais
//! brasileiros (NF-e, NFC-e, CT-e, GTV-e, CT-e Simplificado, CT-e OS, Eventos e e-Financeira).

/*
src/
├── main.rs                  # Ponto de entrada (CLI Runner / Error Boundary)
├── lib.rs                   # Fachada pública e orquestração dos submódulos
├── error.rs                 # Definição global de erros (XmlParserError / XmlParserResult)
├── xml_validation.rs        # Varredura SAX para detecção de tags não mapeadas (-k)
├── example_iso_8859_1.rs    # Testes de leitura e fallback ISO-8859-1
├── example_utf8.rs          # Testes de desserialização UTF-8
├── example_windows_1252.rs  # Testes de leitura e fallback Windows-1252
│
├── cli/                     # INTERFACE DE LINHA DE COMANDO E TERMINAL
│   ├── mod.rs
│   ├── args.rs              # Definição dos argumentos (Clap)
│   ├── nodes.rs             # Inspeção de nós XML de baixo nível (-n)
│   └── progress.rs          # Gerenciamento de barras de progresso (Indicatif)
│
├── core/                    # MOTOR DE REGRAS DE NEGÓCIO E DOMÍNIO FISCAL
│   ├── mod.rs
│   ├── aggregations.rs      # Agrupamentos analíticos por NCM/Tomador
│   ├── docs_fiscais.rs      # Repositório consolidado (DocsFiscais)
│   ├── event.rs             # Vinculação de eventos e cancelamentos
│   ├── graph.rs             # Algoritmo DSU e fecho transitivo CT-e ↔ NF-e
│   ├── information.rs       # Despachante de parsers (Information / StructExtension)
│   ├── key_filter.rs        # Filtro por chaves de acesso de 44 caracteres (-f)
│   └── sampling.rs          # Amostragem estratificada em streaming O(n)
│
├── export/                  # GERADORES DE SAÍDA TABULAR E TABELAS FISCAIS
│   ├── mod.rs
│   ├── cfop.rs              # Tabela oficial CONFAZ de CFOP
│   ├── cst.rs               # Tabela oficial 4.3.3/4.3.4 de CST PIS/COFINS
│   ├── csv.rs               # Gravador otimizado de CSV
│   ├── excel.rs             # Serialização e streaming para XLSX (rust_xlsxwriter)
│   ├── format.rs            # Estilos, fontes e regras visuais de células
│   ├── memory_mode.rs       # Perfis de consumo de RAM (Constant, Low, Memory)
│   ├── natureza.rs          # Tabela 4.3.7 e CFOPs geradores de crédito
│   ├── ncm.rs               # Tabela oficial Siscomex/RFB de NCM
│   └── receita.rs           # Layout padrão Receita Federal (56 colunas)
│
├── pipeline/                # MOTORES DE EXECUÇÃO SOBRE FONTES XML (.zip E .xml)
│   ├── mod.rs
│   ├── memory.rs            # Pipeline integral em memória (máxima velocidade)
│   ├── out_of_core.rs       # Pipeline em disco (-b) com hash sharding + K-way merge
│   ├── spill_codec.rs       # Codec postcard + zstd e iterador de merge sort
│   └── zip_source.rs        # Leitura paralela intra-ZIP e inter-ZIP via pread
│
├── utils/                   # UTILITÁRIOS, TRAITS AUXILIARES E REGEX
│   ├── mod.rs
│   ├── common.rs            # Manipulação de caminhos, datas NaiveDate e I/O
│   ├── group_by_hashmap.rs  # Extensão para agrupamento concorrente via Rayon
│   ├── regex.rs             # Compilação estática de Regex e RegexSet
│   ├── traits.rs            # Encodings XML, sanitização de texto, Option/Slice exts
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
#[cfg(test)]
mod example_iso_8859_1;
#[cfg(test)]
mod example_utf8;
#[cfg(test)]
mod example_windows_1252;
mod export;
mod pipeline;
mod utils;
mod xml_structs;
mod xml_validation;

pub use self::{
    cli::*,
    core::*,
    error::{XmlParserError, XmlParserResult},
    export::*,
    pipeline::*,
    utils::*,
    xml_structs::*,
    xml_validation::*,
};

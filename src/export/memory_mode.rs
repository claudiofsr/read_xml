use clap::ValueEnum;
use rust_xlsxwriter::{Workbook, Worksheet};
use serde::{Deserialize, Serialize};

/// Estratégias de consumo de memória para a geração do documento Excel.
#[derive(Default, ValueEnum, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExcelMemoryMode {
    /// Grava progressivamente os dados diretamente em disco sob demanda.
    ///
    /// Ideal para cenários críticos de hardware, onde o volume de dados excede a
    /// capacidade de memória RAM livre, sacrificando desempenho em favor da estabilidade.
    Constant,

    /// Perfil de baixa memória com uso otimizado de strings compartilhadas.
    ///
    /// Busca um equilíbrio intermediário entre consumo de heap e performance.
    #[default]
    Low,

    /// Executa todo o processamento de formatação e montagem do Workbook em memória RAM.
    ///
    /// Esta abordagem tira proveito de execuções paralelas na thread pool do Rayon,
    /// sendo recomendada quando a performance de escrita é a maior prioridade.
    Memory,
}

impl ExcelMemoryMode {
    /// Indica se o modo selecionado se beneficia do pipeline de execução paralela do Rayon.
    #[inline]
    pub fn is_parallel(self) -> bool {
        matches!(self, Self::Memory)
    }

    /// Cria e associa uma planilha ao Workbook fornecido conforme o perfil de alocação de memória.
    ///
    /// O parâmetro de tempo de vida `'a` garante que a referência mútua devolvida permaneça
    /// válida respeitando a posse interna do `Workbook`.
    pub fn add_worksheet_to_workbook(self, workbook: &mut Workbook) -> &mut Worksheet {
        match self {
            Self::Constant => workbook.add_worksheet_with_constant_memory(),
            Self::Low => workbook.add_worksheet_with_low_memory(),
            Self::Memory => workbook.add_worksheet(),
        }
    }
}

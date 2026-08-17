//! # Detalhes e Agendamento de Entrega
//!
//! Este módulo gerencia as estruturas de dados e regras de desserialização para
//! locais de entrega física de mercadorias (`Entrega`), bem como os perfis de
//! agendamento temporal de frete (datas e horas programadas, intervalos e períodos).

use claudiofsr_lib::StrExtension;
use serde::{Deserialize, Serialize};

/// Dados estruturados do endereço do local de entrega física da carga (`<entrega>`).
///
/// Contém o endereço de destino final da mercadoria quando este difere do endereço
/// cadastral do destinatário principal da Nota Fiscal.
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Entrega {
    /// CNPJ correspondente ao ponto de entrega física.
    #[serde(rename = "CNPJ", default)]
    pub cnpj: Option<String>,

    /// CPF correspondente ao ponto de entrega física se destinado a pessoa física.
    #[serde(rename = "CPF", default)]
    pub cpf: Option<String>,

    /// Razão Social ou Nome do recebedor no local de entrega.
    #[serde(rename = "xNome", default)]
    pub x_nome: Option<String>,

    /// Logradouro do endereço de entrega.
    #[serde(rename = "xLgr", default)]
    pub x_lgr: Option<String>,

    /// Número do endereço de entrega.
    #[serde(rename = "nro", default)]
    pub nro: Option<String>,

    /// Complemento do endereço de entrega.
    #[serde(rename = "xCpl", default)]
    pub x_cpl: Option<String>,

    /// Bairro do endereço de entrega.
    #[serde(rename = "xBairro", default)]
    pub x_bairro: Option<String>,

    /// Código do Município de entrega (utilizar a tabela do IBGE).
    #[serde(rename = "cMun", default)]
    pub c_mun: Option<String>,

    /// Nome descritivo do Município correspondente.
    #[serde(rename = "xMun", default)]
    pub x_mun: Option<String>,

    /// Sigla da Unidade Federativa (UF) de entrega.
    #[serde(rename = "UF", default)]
    pub uf: Option<String>,

    /// Código de Endereçamento Postal (CEP).
    #[serde(rename = "CEP", default)]
    pub cep: Option<String>,

    /// Código identificador do País de entrega.
    #[serde(rename = "cPais", default)]
    pub c_pais: Option<String>,

    /// Nome descritivo do País correspondente.
    #[serde(rename = "xPais", default)]
    pub x_pais: Option<String>,

    /// Telefone cadastrado para contato no local de entrega.
    #[serde(rename = "fone", default)]
    pub fone: Option<String>,

    /// Endereço de correio eletrônico cadastrado.
    #[serde(rename = "email", default)]
    pub email: Option<String>,

    /// Inscrição Estadual (IE) do recebedor se aplicável.
    #[serde(rename = "IE", default)]
    pub ie: Option<String>,

    /// Opção de agendamento de entrega com data programada.
    #[serde(rename = "comData", default)]
    pub com_data: Option<ComData>,

    /// Opção de agendamento de entrega com hora programada.
    #[serde(rename = "comHora", default)]
    pub com_hora: Option<ComHora>,

    /// Opção de agendamento de entrega em intervalo de horas específico.
    #[serde(rename = "noInter", default)]
    pub no_inter: Option<NoInter>,

    /// Opção de agendamento de entrega em um período de datas específico.
    #[serde(rename = "noPeriodo", default)]
    pub no_periodo: Option<NoPeriodo>,

    /// Opção de agendamento de entrega sem data programada definida.
    #[serde(rename = "semData", default)]
    pub sem_data: Option<SemData>,

    /// Opção de agendamento de entrega sem hora programada definida.
    #[serde(rename = "semHora", default)]
    pub sem_hora: Option<SemHora>,
}

impl Entrega {
    /// Retorna o CNPJ formatado caso esteja presente.
    pub fn get_cnpj(&self) -> Option<String> {
        self.cnpj.as_ref().map(|c| c.trim().format_cnpj())
    }

    /// Retorna o CPF formatado caso esteja presente.
    pub fn get_cpf(&self) -> Option<String> {
        self.cpf.as_ref().map(|c| c.trim().format_cpf())
    }

    /// Retorna o nome do Município de entrega limpo e em caixa alta.
    pub fn get_municipio(&self) -> Option<String> {
        self.x_mun.as_ref().map(|m| m.trim().to_uppercase())
    }

    /// Retorna a sigla da Unidade Federativa (UF) de entrega limpa e em caixa alta.
    pub fn get_uf(&self) -> Option<String> {
        self.uf.as_ref().map(|u| u.trim().to_uppercase())
    }
}

/// Dados de agendamento de entrega com data programada definida (`<comData>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ComData {
    /// Texto descritivo contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    /// Data programada da entrega, no formato AAAA-MM-DD.
    #[serde(rename = "dProg", default)]
    pub d_prog: Option<String>,

    /// Tipo de período de data programado (p. ex., manhã, tarde).
    #[serde(rename = "tpPer", default)]
    pub tp_per: Option<String>,
}

/// Dados de agendamento de entrega com hora programada definida (`<comHora>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ComHora {
    /// Texto descritivo contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    /// Hora programada da entrega, no formato HH:MM:SS.
    #[serde(rename = "hProg", default)]
    pub h_prog: Option<String>,

    /// Tipo de horário programado.
    #[serde(rename = "tpHor", default)]
    pub tp_hor: Option<String>,
}

/// Dados de agendamento de entrega em um intervalo de horas específico (`<noInter>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NoInter {
    /// Texto descritivo contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    /// Hora final limite para a realização da entrega.
    #[serde(rename = "hFim", default)]
    pub h_fim: Option<String>,

    /// Hora inicial limite para a realização da entrega.
    #[serde(rename = "hIni", default)]
    pub h_ini: Option<String>,

    /// Tipo de intervalo de horário.
    #[serde(rename = "tpHor", default)]
    pub tp_hor: Option<String>,
}

/// Dados de agendamento de entrega em um período de datas específico (`<noPeriodo>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NoPeriodo {
    /// Texto descritivo contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    /// Data limite final para a realização da entrega.
    #[serde(rename = "dFim", default)]
    pub d_fim: Option<String>,

    /// Data limite inicial para a realização da entrega.
    #[serde(rename = "dIni", default)]
    pub d_ini: Option<String>,

    /// Tipo de período de data.
    #[serde(rename = "tpPer", default)]
    pub tp_per: Option<String>,
}

/// Dados de agendamento de entrega sem data programada definida (`<semData>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SemData {
    /// Texto descritivo contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    /// Tipo de período de data correspondente à entrega.
    #[serde(rename = "tpPer", default)]
    pub tp_per: Option<String>,
}

/// Dados de agendamento de entrega sem hora programada definida (`<semHora>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SemHora {
    /// Texto descritivo contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    /// Tipo de período de horário correspondente à entrega.
    #[serde(rename = "tpHor", default)]
    pub tp_hor: Option<String>,
}

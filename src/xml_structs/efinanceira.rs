//! # Processamento de XML de e-Financeira (SPED)
//!
//! Este módulo gerencia a desserialização e estruturação dos dados de movimentações
//! financeiras reportadas ao SPED (e-Financeira) de acordo com o leiaute oficial.
//!
//! Para gerar instâncias de teste e validar a estrutura a partir do XSD original:
//!
//! ```console
//! wget http://sped.rfb.gov.br/estatico/59/791D5602EF73CE796AD62E07FA8FF6500CBC99/evtMovOpFin-v1_2_1.xsd
//! xsd2inst evtMovOpFin-v1_2_1.xsd -name eFinanceira -dl > evtMovOpFin-v1_2_1.xml
//! read-xml -s evtMovOpFin-v1_2_1.xml > evtMovOpFin-v1_2_1.rs
//! ```

/*
Generates XML file from Multiple XML schemas (xsd files).

This feature requires Apache XMLBeans.
https://xmlbeans.apache.org/docs/2.0.0/guide/tools.html#xsd2inst
xsd2inst (Schema to Instance Tool)
Prints an XML instance from the specified global element using the specified schema.

Download schema xsd file:
wget http://sped.rfb.gov.br/estatico/59/791D5602EF73CE796AD62E07FA8FF6500CBC99/evtMovOpFin-v1_2_1.xsd

XSD to XML:
/home/claudio/Downloads/XMLBeans/xmlbeans-5.2.0/bin/xsd2inst evtMovOpFin-v1_2_1.xsd -name eFinanceira -dl > evtMovOpFin-v1_2_1.xml

Converter XML file to Rust struct:
read-xml -s evtMovOpFin-v1_2_1.xml > evtMovOpFin-v1_2_1.rs
*/

use chrono::NaiveDate;
use claudiofsr_lib::StrExtension;
use rust_xlsxwriter::serialize_option_datetime_to_excel;
use serde::{Deserialize, Serialize};
use struct_iterable::Iterable;

use crate::{
    Arguments, GetID, InfoExtension, Information, OptExt, StructExtension,
    get_naive_date_from_yyyy_mm_dd, get_naive_date_from_yyyymm,
    xml_structs::{agente::format_cnpj_safe, assinaturas::Signature},
};

/// Macro auxiliar de exemplo para extração de nomes de campos de structs via reflexão.
#[allow(unused_macros)]
macro_rules! my_macro {
    (pub struct $name:ident {
        $(pub $field_name:ident: $field_type:ty,)*
    }) => {
        pub struct $name {
            pub $($field_name: $field_type,)*
        }

        impl $name {
            /// Retorna os nomes dos campos declarados na estrutura como strings estáticas.
            fn get_field_names() -> Vec<&'static str> {
                vec![$(stringify!($field_name)),*]
            }
        }
    }
}

/// Representação intermediária e consolidada de uma conta financeira reportada.
///
/// Consolida metadados da entidade declarante, informações cadastrais do titular
/// e balanços de saldos (totais de créditos e débitos).
#[derive(Debug, Default, Serialize, Deserialize, Clone, Iterable)]
pub struct InfoEFinanceira {
    /// Identificador único gerado para o evento.
    #[serde(rename = "Identificação da Entidade Declarante", default)]
    pub id: Option<String>,

    /// Indicador de retificação do evento (`ideEvento/indRetificacao`).
    #[serde(rename = "Indicador de Retificação", default)]
    pub ind_retificacao: Option<String>,

    /// Número do recibo anterior em caso de retificação (`ideEvento/nrRecibo`).
    #[serde(rename = "Nº do Recibo", default)]
    pub nr_recibo: Option<String>,

    /// CNPJ formatado da instituição financeira declarante.
    #[serde(rename = "CNPJ da Entidade Declarante", default)]
    pub cnpj_do_declarante: Option<String>,

    /// Número de Identificação Fiscal (NIF) ou CNPJ/CPF do declarado.
    #[serde(rename = "Identificação do declarado", default)]
    pub ni_do_declarado: Option<String>,

    /// Razão social ou nome civil do declarado titular da conta.
    #[serde(rename = "Nome do declarado", default)]
    pub nome_declarado: Option<String>,

    /// Data correspondente ao fechamento do mês caixa reportado.
    #[serde(
        rename = "Ano/Mês Caixa Reportado",
        serialize_with = "serialize_option_datetime_to_excel",
        default
    )]
    pub ano_mes_caixa: Option<NaiveDate>,

    /// Tipo da conta financeira (`infoConta/tpConta`).
    #[serde(rename = "Tipo da Conta", default)]
    pub tp_conta: Option<String>,

    /// Subtipo da conta financeira (`infoConta/subTpConta`).
    #[serde(rename = "Subtipo da Conta", default)]
    pub sub_tp_conta: Option<String>,

    /// Número identificador da conta financeira.
    #[serde(rename = "Nº da Conta", default)]
    pub num_conta: Option<String>,

    /// Data de abertura da conta (`infoConta/dtAberturaConta`).
    #[serde(
        rename = "Data de Abertura da Conta",
        serialize_with = "serialize_option_datetime_to_excel",
        default
    )]
    pub dt_abertura_conta: Option<NaiveDate>,

    /// Forma de abertura da conta (`infoConta/formaAberturaConta`).
    #[serde(rename = "Forma de Abertura da Conta", default)]
    pub forma_abertura_conta: Option<String>,

    /// Data de encerramento da conta (`infoConta/dtEncerramentoConta`).
    #[serde(
        rename = "Data de Encerramento da Conta",
        serialize_with = "serialize_option_datetime_to_excel",
        default
    )]
    pub dt_encerramento_conta: Option<NaiveDate>,

    /// Moeda da conta (`infoConta/moeda`).
    #[serde(rename = "Moeda", default)]
    pub moeda: Option<String>,

    /// Lista consolidada de países onde o titular possui domicílio fiscal.
    #[serde(rename = "País Reportado", default)]
    pub pais_reportado: Option<String>,

    /// Acumulado anual/mensal correspondente a créditos na conta.
    #[serde(rename = "Balanço da Conta: Total de Créditos", default)]
    pub tot_creditos: Option<f64>,

    /// Acumulado anual/mensal correspondente a débitos na conta.
    #[serde(rename = "Balanço da Conta: Total de Débitos", default)]
    pub tot_debitos: Option<f64>,

    /// Acumulado de créditos de mesma titularidade (`totCreditosMesmaTitularidade`).
    #[serde(
        rename = "Balanço da Conta: Total de Créditos Mesma Titularidade",
        default
    )]
    pub tot_creditos_mesma_titularidade: Option<f64>,

    /// Acumulado de débitos de mesma titularidade (`totDebitosMesmaTitularidade`).
    #[serde(
        rename = "Balanço da Conta: Total de Débitos Mesma Titularidade",
        default
    )]
    pub tot_debitos_mesma_titularidade: Option<f64>,

    /// Saldo no último dia útil do período (`BalancoConta/vlrUltDia`).
    #[serde(rename = "Balanço da Conta: Valor no Último Dia", default)]
    pub vlr_ult_dia: Option<f64>,
}

impl InfoExtension for InfoEFinanceira {}

impl
    GetID<(
        Option<String>,
        Option<String>,
        Option<NaiveDate>,
        Option<String>,
        Option<String>,
    )> for InfoEFinanceira
{
    fn get_id(
        &self,
    ) -> (
        Option<String>,
        Option<String>,
        Option<NaiveDate>,
        Option<String>,
        Option<String>,
    ) {
        (
            self.cnpj_do_declarante.clone(),
            self.ni_do_declarado.clone(),
            self.ano_mes_caixa,
            self.num_conta.clone(),
            self.id.clone(),
        )
    }
}

crate::impl_sanitize_text!(
    InfoEFinanceira,
    id,
    ind_retificacao,
    nr_recibo,
    cnpj_do_declarante,
    ni_do_declarado,
    nome_declarado,
    tp_conta,
    sub_tp_conta,
    num_conta,
    forma_abertura_conta,
    moeda,
    pais_reportado,
);

impl InfoEFinanceira {
    /// Calcula o comprimento em caracteres de cada campo para formatação em planilha Excel.
    pub fn get_lengths(&self) -> Vec<usize> {
        vec![
            self.id.count(),
            self.cnpj_do_declarante.count(),
            self.ni_do_declarado.count(),
            self.nome_declarado.count(),
            self.ano_mes_caixa.count(),
            self.num_conta.count(),
            self.pais_reportado.count(),
            self.tot_creditos
                .map(|val| val.to_string().chars_count())
                .unwrap_or(0),
            self.tot_debitos
                .map(|val| val.to_string().chars_count())
                .unwrap_or(0),
        ]
    }
}

/// Estrutura correspondente à raiz do leiaute da e-Financeira (`<eFinanceira>`).
#[derive(Debug, Serialize, Deserialize)]
pub struct EFinanceira {
    // =========================================================================
    // 1. ATRIBUTOS DO NÓ XML (Mapeados no topo com default)
    // =========================================================================
    /// Versão declarada do leiaute.
    #[serde(rename = "@versao", default)]
    pub versao: Option<String>,

    /// Namespace XML correspondente.
    #[serde(rename = "@xmlns", default)]
    pub xmlns: Option<String>,

    // =========================================================================
    // 2. CAMPOS OPCIONAIS (Mapeados no meio com default)
    // =========================================================================
    /// Texto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    // =========================================================================
    // 3. CAMPOS OBRIGATÓRIOS (Mapeados na base sem default)
    // =========================================================================
    /// Evento principal de informações de movimentações financeiras (`evtMovOpFin`).
    #[serde(rename = "evtMovOpFin", default)]
    pub evt_mov_op_fin: Option<EvtMovOpFin>,

    /// Evento anual de movimentações financeiras (`evtMovOpFinAnual`).
    #[serde(rename = "evtMovOpFinAnual", default)]
    pub evt_mov_op_fin_anual: Option<EvtMovOpFin>,

    /// Lote síncrono de eventos (`loteEventos`).
    #[serde(rename = "loteEventos", default)]
    pub lote_eventos: Option<LoteEventos>,

    /// Lote assíncrono de eventos (`loteEventosAssincrono`).
    #[serde(rename = "loteEventosAssincrono", default)]
    pub lote_eventos_assincrono: Option<LoteEventos>,

    /// Bloco contendo a assinatura digital ICP-Brasil.
    #[serde(rename = "Signature", default)]
    pub signature: Option<Signature>,
}

/// Envelope de lote síncrono/assíncrono da e-Financeira (`<loteEventos>` / `<loteEventosAssincrono>`).
#[derive(Debug, Serialize, Deserialize)]
pub struct LoteEventos {
    #[serde(rename = "cnpjDeclarante", default)]
    pub cnpj_declarante: Option<String>,
    #[serde(rename = "eventos", default)]
    pub eventos: Option<LoteEventosLista>,
}

/// Lista de eventos dentro de um lote da e-Financeira (`<eventos>`).
#[derive(Debug, Serialize, Deserialize)]
pub struct LoteEventosLista {
    #[serde(rename = "evento", default)]
    pub evento: Vec<LoteEventoItem>,
}

/// Item de evento encapsulado em um lote (`<evento>`).
#[derive(Debug, Serialize, Deserialize)]
pub struct LoteEventoItem {
    #[serde(rename = "@id", alias = "@Id", default)]
    pub id: Option<String>,
    #[serde(rename = "eFinanceira", default)]
    pub e_financeira: Option<Box<EFinanceira>>,
}

impl StructExtension for EFinanceira {
    /// Converte os nós da e-Financeira para uma coleção de informações consolidadas.
    fn get_information(&self, xml_path: &std::path::Path, arguments: &Arguments) -> Information {
        if arguments.verbose {
            println!("eFinanceira xml_path: {xml_path:?}");
            println!("eFinanceira: {self:#?}\n");
        }
        let infos = self.get_infos();
        if infos.is_empty() && !self.has_movimentacao() {
            Information::None
        } else {
            Information::EFinanceira(infos)
        }
    }
}

impl EFinanceira {
    fn main_evt(&self) -> Option<&EvtMovOpFin> {
        self.evt_mov_op_fin
            .as_ref()
            .or(self.evt_mov_op_fin_anual.as_ref())
    }

    fn has_movimentacao(&self) -> bool {
        self.main_evt().is_some()
            || self.lote_eventos.is_some()
            || self.lote_eventos_assincrono.is_some()
    }

    /// Retorna o identificador gerado para o lote de movimentações.
    pub fn get_id(&self) -> Option<String> {
        self.main_evt().and_then(|e| e.id.clone())
    }

    /// Retorna o CNPJ formatado do declarante.
    pub fn get_cnpj_do_declarante(&self) -> Option<String> {
        self.main_evt()
            .and_then(EvtMovOpFin::get_cnpj_do_declarante)
    }

    /// Retorna o Número de Identificação Fiscal (CPF, CNPJ ou NIF estrangeiro) formatado.
    pub fn get_ni_do_declarado(&self) -> Option<String> {
        self.main_evt().and_then(EvtMovOpFin::get_ni_do_declarado)
    }

    /// Retorna o nome civil ou razão social do declarado sem quebras de linha.
    pub fn get_nome_declarado(&self) -> Option<String> {
        self.main_evt().and_then(EvtMovOpFin::get_nome_declarado)
    }

    /// Obtém a data correspondente ao mês-caixa de referência.
    pub fn get_ano_mes(&self) -> Option<NaiveDate> {
        self.main_evt().and_then(EvtMovOpFin::get_ano_mes)
    }

    /// Consolida todos os metadados e balanços de contas em uma coleção de `InfoEFinanceira`.
    pub fn get_infos(&self) -> Vec<InfoEFinanceira> {
        let mut out = Vec::new();
        if let Some(evt) = &self.evt_mov_op_fin {
            out.extend(evt.get_infos());
        }
        if let Some(evt_anual) = &self.evt_mov_op_fin_anual {
            out.extend(evt_anual.get_infos());
        }
        for lote in [&self.lote_eventos, &self.lote_eventos_assincrono]
            .into_iter()
            .flatten()
        {
            if let Some(lista) = &lote.eventos {
                for item in &lista.evento {
                    if let Some(ef) = &item.e_financeira {
                        out.extend(ef.get_infos());
                    }
                }
            }
        }
        out
    }
}

impl EvtMovOpFin {
    pub fn get_cnpj_do_declarante(&self) -> Option<String> {
        self.ide_declarante
            .cnpj_declarante
            .as_deref()
            .map(str::trim)
            .filter(|c| !c.is_empty())
            .map(format_cnpj_safe)
    }

    pub fn get_ni_do_declarado(&self) -> Option<String> {
        let ni = self.ide_declarado.nideclarado.as_deref()?.trim();
        if ni.is_empty() {
            return None;
        }
        let alnum: String = ni.chars().filter(char::is_ascii_alphanumeric).collect();
        if alnum.len() == 14 && alnum.bytes().any(|b| b.is_ascii_alphabetic()) {
            return Some(format_cnpj_safe(&alnum));
        }
        let digits = ni.remove_non_digits();
        match digits.len() {
            11 => Some(digits.format_cpf()),
            14 => Some(format_cnpj_safe(&digits)),
            _ => Some(ni.to_string()),
        }
    }

    pub fn get_nome_declarado(&self) -> Option<String> {
        self.ide_declarado
            .nome_declarado
            .as_deref()
            .and_then(|nome| {
                let clean = crate::strip_line_breaks(nome);
                (!clean.is_empty()).then_some(clean)
            })
    }

    fn get_contas(&self) -> Vec<&Conta> {
        self.mes_caixa
            .mov_op_fin
            .as_ref()
            .and_then(|m| m.conta.as_ref())
            .map_or_else(Vec::new, |vec| vec.iter().collect())
    }

    pub fn get_ano_mes(&self) -> Option<NaiveDate> {
        self.mes_caixa.get_date_ano_mes_caixa()
    }

    pub fn get_infos(&self) -> Vec<InfoEFinanceira> {
        let id = self.id.clone();
        let ind_retificacao = self.ide_evento.ind_retificacao.get_not_empty();
        let nr_recibo = self.ide_evento.nr_recibo.get_not_empty();
        let cnpj_do_declarante = self.get_cnpj_do_declarante();
        let ni_do_declarado = self.get_ni_do_declarado();
        let nome_declarado = self.get_nome_declarado();
        let ano_mes_caixa = self.get_ano_mes();

        self.get_contas()
            .into_iter()
            .map(|conta| {
                let balanco = conta.info_conta.balanco_conta.as_ref();
                InfoEFinanceira {
                    id: id.clone(),
                    ind_retificacao: ind_retificacao.clone(),
                    nr_recibo: nr_recibo.clone(),
                    cnpj_do_declarante: cnpj_do_declarante.clone(),
                    ni_do_declarado: ni_do_declarado.clone(),
                    nome_declarado: nome_declarado.clone(),
                    ano_mes_caixa,
                    tp_conta: conta.info_conta.tp_conta.get_not_empty(),
                    sub_tp_conta: conta.info_conta.sub_tp_conta.get_not_empty(),
                    num_conta: conta.info_conta.num_conta.get_not_empty(),
                    dt_abertura_conta: get_naive_date_from_yyyy_mm_dd(
                        &conta.info_conta.dt_abertura_conta,
                    ),
                    forma_abertura_conta: conta.info_conta.forma_abertura_conta.get_not_empty(),
                    dt_encerramento_conta: get_naive_date_from_yyyy_mm_dd(
                        &conta.info_conta.dt_encerramento_conta,
                    ),
                    moeda: conta.info_conta.moeda.get_not_empty(),
                    pais_reportado: conta.info_conta.get_paises(),
                    tot_creditos: balanco.and_then(|b| b.tot_creditos.to_float64()),
                    tot_debitos: balanco.and_then(|b| b.tot_debitos.to_float64()),
                    tot_creditos_mesma_titularidade: balanco
                        .and_then(|b| b.tot_creditos_mesma_titularidade.to_float64()),
                    tot_debitos_mesma_titularidade: balanco
                        .and_then(|b| b.tot_debitos_mesma_titularidade.to_float64()),
                    vlr_ult_dia: balanco.and_then(|b| b.vlr_ult_dia.to_float64()),
                }
            })
            .collect()
    }
}

/// Bloco contendo os dados do evento de movimentações financeiras (`<evtMovOpFin>`).
#[derive(Debug, Serialize, Deserialize)]
pub struct EvtMovOpFin {
    // =========================================================================
    // 1. ATRIBUTOS DO NÓ XML
    // =========================================================================
    /// Identificador do evento.
    #[serde(rename = "@id", alias = "@Id", default)]
    pub id: Option<String>,

    // =========================================================================
    // 2. CAMPOS OPCIONAIS
    // =========================================================================
    /// Texto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    // =========================================================================
    // 3. CAMPOS OBRIGATÓRIOS
    // =========================================================================
    /// Identificação do Evento.
    #[serde(rename = "ideEvento")]
    pub ide_evento: IdeEvento,

    /// Identificação da Entidade Declarante.
    #[serde(rename = "ideDeclarante")]
    pub ide_declarante: IdeDeclarante,

    /// Identificação do Declarado.
    #[serde(rename = "ideDeclarado")]
    pub ide_declarado: IdeDeclarado,

    /// Identificação do Mês/Caixa (`mesCaixa` ou `caixaAnual` em `evtMovOpFinAnual`).
    #[serde(rename = "mesCaixa", alias = "caixaAnual")]
    pub mes_caixa: MesCaixa,
}

/// Identificação do Evento reportado (`<ideEvento>`).
#[derive(Debug, Serialize, Deserialize)]
pub struct IdeEvento {
    /// Indicador de retificação (p. ex., retificador).
    #[serde(rename = "indRetificacao", default)]
    pub ind_retificacao: Option<String>,

    /// Número do recibo anterior em caso de retificação.
    #[serde(rename = "nrRecibo", default)]
    pub nr_recibo: Option<String>,

    /// Ambiente de destino (Produção/Homologação).
    #[serde(rename = "tpAmb", default)]
    pub tp_amb: Option<String>,

    /// Aplicação que realizou a emissão do evento.
    #[serde(rename = "aplicEmi", default)]
    pub aplic_emi: Option<String>,

    /// Versão do aplicativo emissor.
    #[serde(rename = "verAplic", default)]
    pub ver_aplic: Option<String>,

    /// Texto bruto contido no nó.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// Identificação da Entidade Declarante (`<ideDeclarante>`).
#[derive(Debug, Serialize, Deserialize)]
pub struct IdeDeclarante {
    /// CNPJ da Entidade Declarante.
    #[serde(rename = "cnpjDeclarante", default)]
    pub cnpj_declarante: Option<String>,

    /// Texto bruto contido no nó.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// Identificação cadastral do Declarado (`<ideDeclarado>`).
#[derive(Debug, Serialize, Deserialize)]
pub struct IdeDeclarado {
    /// Tipo de Número de Identificação (CNPJ/CPF, etc.).
    #[serde(rename = "tpNI", default)]
    pub tp_ni: Option<String>,

    /// Tipo de titular declarado.
    #[serde(rename = "tpDeclarado", default)]
    pub tp_declarado: Option<Vec<String>>,

    /// Número de identificação fiscal (CPF/CNPJ/NIF).
    #[serde(rename = "NIDeclarado", default)]
    pub nideclarado: Option<String>,

    /// Nome cadastral completo do declarado.
    #[serde(rename = "NomeDeclarado", default)]
    pub nome_declarado: Option<String>,

    /// Tipo de nome declarado.
    #[serde(rename = "tpNomeDeclarado", default)]
    pub tp_nome_declarado: Option<String>,

    /// Data de nascimento para pessoas físicas.
    #[serde(rename = "DataNasc", default)]
    pub data_nasc: Option<String>,

    /// Endereço informado de forma livre.
    #[serde(rename = "EnderecoLivre", default)]
    pub endereco_livre: Option<String>,

    /// Tipo de endereço (p. ex., residencial).
    #[serde(rename = "tpEndereco", default)]
    pub tp_endereco: Option<String>,

    /// Texto bruto contido no nó.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    /// Números de Identificação Fiscal internacionais se cadastrados.
    #[serde(rename = "NIF", default)]
    pub nif: Option<Vec<IdeDeclaradoNif>>,

    /// Nomes alternativos do declarado.
    #[serde(rename = "NomeOutros", default)]
    pub nome_outros: Option<NifNomeOutros>,

    /// Dados de nascimento cadastrais adicionais.
    #[serde(rename = "InfoNascimento", default)]
    pub info_nascimento: Option<NomePjInfoNascimento>,

    /// Declaração própria CRS (`inDeclaracaoPropriaCRS`).
    #[serde(rename = "inDeclaracaoPropriaCRS", default)]
    pub in_declaracao_propria_crs: Option<String>,

    /// Motivo de ausência de TIN (`IndRazaoFaltaTIN`).
    #[serde(rename = "IndRazaoFaltaTIN", default)]
    pub ind_razao_falta_tin: Option<String>,

    /// Endereço principal estruturado (`<Endereco>`, leiautes v1.3.0+).
    #[serde(rename = "Endereco", default)]
    pub endereco: Option<EnderecoEfin>,

    /// País correspondente ao endereço fiscal.
    #[serde(rename = "PaisEndereco", default)]
    pub pais_endereco: Option<PaisNascPaisEndereco>,

    /// Endereços internacionais alternativos estruturados.
    #[serde(rename = "EnderecoOutros", default)]
    pub endereco_outros: Option<Vec<PaisEnderecoEnderecoOutros>>,

    /// Países adicionais de residência fiscal.
    #[serde(rename = "paisResid", default)]
    pub pais_resid: Option<Vec<EnderecoPaisResid>>,

    /// Países correspondentes às nacionalidades adicionais do titular.
    #[serde(rename = "PaisNacionalidade", default)]
    pub pais_nacionalidade: Option<Vec<PaisResidPaisNacionalidade>>,

    /// Lista de proprietários beneficiários associados.
    #[serde(rename = "Proprietarios", default)]
    pub proprietarios: Option<Vec<Proprietarios>>,
}

/// Identificação Fiscal Internacional (`<NIF>`).
#[derive(Debug, Serialize, Deserialize)]
pub struct IdeDeclaradoNif {
    /// Número identificador internacional.
    #[serde(rename = "NumeroNIF", default)]
    pub numero_nif: Option<String>,

    /// País de emissão do NIF correspondente.
    #[serde(rename = "PaisEmissaoNIF", default)]
    pub pais_emissao_nif: Option<String>,

    /// Tipo de NIF adotado.
    #[serde(rename = "tpNIF", default)]
    pub tp_nif: Option<String>,

    /// Texto bruto contido no nó.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// Bloco contendo nomes alternativos do declarado (`<NomeOutros>`).
#[derive(Debug, Serialize, Deserialize)]
pub struct NifNomeOutros {
    /// Texto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    /// Nome estruturado de Pessoa Física.
    #[serde(rename = "NomePF", default)]
    pub nome_pf: Option<NifNomeOutrosNomePf>,

    /// Nome estruturado de Pessoa Jurídica.
    #[serde(rename = "NomePJ", default)]
    pub nome_pj: Option<NomePj>,
}

/// Dados estruturados de nome alternativo de Pessoa Física (`<NomePF>`).
#[derive(Debug, Serialize, Deserialize)]
pub struct NifNomeOutrosNomePf {
    /// Tipo de nome associado.
    #[serde(rename = "tpNome", default)]
    pub tp_nome: Option<String>,

    /// Precedência de títulos do nome.
    #[serde(rename = "PrecTitulo", default)]
    pub prec_titulo: Option<String>,

    /// Lista de títulos vinculados.
    #[serde(rename = "Titulo", default)]
    pub titulo: Vec<String>,

    /// Identificadores adicionais de geração.
    #[serde(rename = "IdGeracao", default)]
    pub id_geracao: Vec<String>,

    /// Lista de sufixos vinculados.
    #[serde(rename = "Sufixo", default)]
    pub sufixo: Vec<String>,

    /// Indicador genérico de sufixo.
    #[serde(rename = "GenSufixo", default)]
    pub gen_sufixo: Option<String>,

    /// Texto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    /// Estrutura para primeiro nome.
    #[serde(rename = "PrimeiroNome", default)]
    pub primeiro_nome: Option<PrimeiroNome>,

    /// Estrutura para nomes intermediários.
    #[serde(rename = "MeioNome", default)]
    pub meio_nome: Vec<NomeDoMeio>,

    /// Estrutura para prefixo de nome.
    #[serde(rename = "PrefixoNome", default)]
    pub prefixo_nome: Option<PrefixoDoNome>,

    /// Estrutura para último nome/sobrenome.
    #[serde(rename = "UltimoNome", default)]
    pub ultimo_nome: Option<EstruturaNome>,
}

/// Primeiro nome estruturado (`<PrimeiroNome>`).
#[derive(Debug, Serialize, Deserialize)]
pub struct PrimeiroNome {
    /// Tipo cadastrado.
    #[serde(rename = "Tipo", default)]
    pub tipo: Option<String>,

    /// Nome correspondente.
    #[serde(rename = "Nome", default)]
    pub nome: Option<String>,

    /// Texto bruto contido no nó.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// Nome intermediário estruturado (`<MeioNome>`).
#[derive(Debug, Serialize, Deserialize)]
pub struct NomeDoMeio {
    /// Tipo cadastrado.
    #[serde(rename = "Tipo", default)]
    pub tipo: Option<String>,

    /// Nome correspondente.
    #[serde(rename = "Nome", default)]
    pub nome: Option<String>,

    /// Texto bruto contido no nó.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// Prefixo estruturado de nome (`<PrefixoNome>`).
#[derive(Debug, Serialize, Deserialize)]
pub struct PrefixoDoNome {
    /// Tipo cadastrado.
    #[serde(rename = "Tipo", default)]
    pub tipo: Option<String>,

    /// Prefixo correspondente.
    #[serde(rename = "Nome", default)]
    pub nome: Option<String>,

    /// Texto bruto contido no nó.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// Sobrenome estruturado (`<UltimoNome>`).
#[derive(Debug, Serialize, Deserialize)]
pub struct EstruturaNome {
    /// Tipo cadastrado.
    #[serde(rename = "Tipo", default)]
    pub tipo: Option<String>,

    /// Nome correspondente.
    #[serde(rename = "Nome", default)]
    pub nome: Option<String>,

    /// Texto bruto contido no nó.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// Dados de identificação de Pessoa Jurídica (`<NomePJ>`).
#[derive(Debug, Serialize, Deserialize)]
pub struct NomePj {
    /// Tipo de nome corporativo.
    #[serde(rename = "tpNome", default)]
    pub tp_nome: Option<String>,

    /// Nome ou Razão Social correspondente.
    #[serde(rename = "Nome", default)]
    pub nome: Option<String>,

    /// Texto bruto contido no nó.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// Dados correspondentes ao nascimento de Pessoa Física (`<InfoNascimento>`).
#[derive(Debug, Serialize, Deserialize)]
pub struct NomePjInfoNascimento {
    /// Município de nascimento.
    #[serde(rename = "Municipio", default)]
    pub municipio: Option<String>,

    /// Bairro de nascimento.
    #[serde(rename = "Bairro", default)]
    pub bairro: Option<String>,

    /// Texto bruto contido no nó.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    /// País correspondente ao nascimento.
    #[serde(rename = "PaisNasc")]
    pub pais_nasc: NomePjInfoNascimentoPaisNasc,
}

/// País de nascimento cadastrado (`<PaisNasc>`).
#[derive(Debug, Serialize, Deserialize)]
pub struct NomePjInfoNascimentoPaisNasc {
    /// Nome de identificação do país.
    #[serde(rename = "Pais", default)]
    pub pais: Option<String>,

    /// Nome antigo associado se aplicável.
    #[serde(rename = "AntigoNomePais", default)]
    pub antigo_nome_pais: Option<String>,

    /// Texto bruto contido no nó.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// Detalhes de país de endereço cadastrados (`<PaisEndereco>`).
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct PaisNascPaisEndereco {
    /// Sigla identificadora do País.
    #[serde(rename = "Pais", default)]
    pub pais: Option<String>,

    /// Texto bruto contido no nó.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// Endereço estruturado internacional alternativo (`<EnderecoOutros>`).
#[derive(Debug, Serialize, Deserialize)]
pub struct PaisEnderecoEnderecoOutros {
    /// Tipo de endereço associado.
    #[serde(rename = "tpEndereco", default)]
    pub tp_endereco: Option<String>,

    /// País correspondente.
    #[serde(rename = "Pais", default)]
    pub pais: Option<String>,

    /// Endereço preenchido em texto livre.
    #[serde(rename = "EnderecoLivre", default)]
    pub endereco_livre: Option<String>,

    /// Texto bruto contido no nó.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    /// Estrutura secundária detalhada de endereço se aplicável.
    #[serde(rename = "EnderecoEstrutura", default)]
    pub endereco_estrutura: Option<EnderecoV2>,
}

/// Bloco de endereço unificado (`<Endereco>` - leiautes v1.3.0+).
#[derive(Debug, Serialize, Deserialize)]
pub struct EnderecoEfin {
    #[serde(rename = "tpEndereco", default)]
    pub tp_endereco: Option<String>,
    #[serde(rename = "EnderecoLivre", default)]
    pub endereco_livre: Option<String>,
    #[serde(rename = "EnderecoEstrutura", default)]
    pub endereco_estrutura: Option<EnderecoV2>,
    #[serde(rename = "Pais", default)]
    pub pais: Option<String>,
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// Bloco secundário de Endereço estruturado (`<EnderecoEstrutura>`).
#[derive(Debug, Serialize, Deserialize)]
pub struct EnderecoV2 {
    /// Logradouro direto (v1.3.0+).
    #[serde(rename = "Logradouro", default)]
    pub logradouro: Option<String>,
    /// Número direto (v1.3.0+).
    #[serde(rename = "Numero", default)]
    pub numero: Option<String>,
    /// Complemento direto (v1.3.0+).
    #[serde(rename = "Complemento", default)]
    pub complemento: Option<String>,
    /// Andar direto (v1.3.0+).
    #[serde(rename = "Andar", default)]
    pub andar: Option<String>,
    /// Bairro direto (v1.3.0+).
    #[serde(rename = "Bairro", default)]
    pub bairro: Option<String>,
    /// Caixa postal direta (v1.3.0+).
    #[serde(rename = "CaixaPostal", default)]
    pub caixa_postal: Option<String>,

    /// Endereço estruturado em texto livre.
    #[serde(rename = "EnderecoLivre", default)]
    pub endereco_livre: Option<String>,

    /// Código de Endereçamento Postal (CEP).
    #[serde(rename = "CEP", default)]
    pub cep: Option<String>,

    /// Município correspondente.
    #[serde(rename = "Municipio", default)]
    pub municipio: Option<String>,

    /// Unidade Federativa.
    #[serde(rename = "UF", default)]
    pub uf: Option<String>,

    /// Texto bruto contido no nó.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    /// Terceiro nível estruturado de endereço (opcional em v1.3.0+).
    #[serde(rename = "Endereco", default)]
    pub endereco: Option<EnderecoV3>,
}

/// Nível de detalhes finais de Endereço estruturado (`<Endereco>`).
#[derive(Debug, Serialize, Deserialize)]
pub struct EnderecoV3 {
    /// Logradouro correspondente.
    #[serde(rename = "Logradouro", default)]
    pub logradouro: Option<String>,

    /// Número do logradouro.
    #[serde(rename = "Numero", default)]
    pub numero: Option<String>,

    /// Complemento se aplicável.
    #[serde(rename = "Complemento", default)]
    pub complemento: Option<String>,

    /// Andar se aplicável.
    #[serde(rename = "Andar", default)]
    pub andar: Option<String>,

    /// Bairro correspondente.
    #[serde(rename = "Bairro", default)]
    pub bairro: Option<String>,

    /// Caixa Postal cadastrada.
    #[serde(rename = "CaixaPostal", default)]
    pub caixa_postal: Option<String>,

    /// Texto bruto contido no nó.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// Bloco correspondente ao país de residência tributária (`<paisResid>`).
#[derive(Debug, Serialize, Deserialize)]
pub struct EnderecoPaisResid {
    /// Sigla identificadora do País.
    #[serde(rename = "Pais", default)]
    pub pais: Option<String>,

    /// Texto bruto contido no nó.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// Bloco correspondente à nacionalidade fiscal (`<PaisNacionalidade>`).
#[derive(Debug, Serialize, Deserialize)]
pub struct PaisResidPaisNacionalidade {
    /// Sigla identificadora do País.
    #[serde(rename = "Pais", default)]
    pub pais: Option<String>,

    /// Texto bruto contido no nó.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// Dados estruturados correspondentes a Proprietários Beneficiários (`<Proprietarios>`).
#[derive(Debug, Serialize, Deserialize)]
pub struct Proprietarios {
    /// Tipo de identificação nacional (tpNI).
    #[serde(rename = "tpNI", default)]
    pub tp_ni: Option<String>,

    /// Número identificador civil do proprietário.
    #[serde(rename = "NIProprietario", default)]
    pub niproprietario: Option<String>,

    /// Tipo correspondente ao proprietário.
    #[serde(rename = "tpProprietario", default)]
    pub tp_proprietario: Option<String>,

    /// Nome cadastral completo.
    #[serde(rename = "Nome", default)]
    pub nome: Option<String>,

    /// Tipo de nome.
    #[serde(rename = "tpNome", default)]
    pub tp_nome: Option<String>,

    /// Endereço em texto livre.
    #[serde(rename = "EnderecoLivre", default)]
    pub endereco_livre: Option<String>,

    /// Tipo de endereço associado.
    #[serde(rename = "tpEndereco", default)]
    pub tp_endereco: Option<String>,

    /// Data de nascimento cadastrada.
    #[serde(rename = "DataNasc", default)]
    pub data_nasc: Option<String>,

    /// Texto bruto contido no nó.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    /// Declaração própria CRS (`inDeclaracaoPropriaCRS`).
    #[serde(rename = "inDeclaracaoPropriaCRS", default)]
    pub in_declaracao_propria_crs: Option<String>,

    /// Motivo de ausência de TIN (`IndRazaoFaltaTIN`).
    #[serde(rename = "IndRazaoFaltaTIN", default)]
    pub ind_razao_falta_tin: Option<String>,

    /// Números identificadores fiscais estruturados (0..N ocorrências).
    #[serde(rename = "NIF", default)]
    pub nif: Option<Vec<ProprietariosNif>>,

    /// Lista de nomes secundários do proprietário.
    #[serde(rename = "NomeOutros", default)]
    pub nome_outros: Option<Vec<NifNomeOutros>>,

    /// Endereço estruturado do proprietário (v1.3.0+).
    #[serde(rename = "Endereco", default)]
    pub endereco: Option<EnderecoEfin>,

    /// Endereço fiscal estruturado do proprietário.
    #[serde(rename = "PaisEndereco", default)]
    pub pais_endereco: Option<UltimoNomePaisEndereco>,

    /// Endereços internacionais alternativos adicionais.
    #[serde(rename = "EnderecoOutros", default)]
    pub endereco_outros: Option<Vec<PaisEnderecoEnderecoOutros>>,

    /// Países de residência tributária.
    #[serde(rename = "paisResid", default)]
    pub pais_resid: Option<Vec<EnderecoPaisResid>>,

    /// Países de nacionalidade declarados do proprietário.
    #[serde(rename = "PaisNacionalidade", default)]
    pub pais_nacionalidade: Option<Vec<PaisResidPaisNacionalidade>>,

    /// Informações adicionais do local de nascimento.
    #[serde(rename = "InfoNascimento", default)]
    pub info_nascimento: Option<LocalNascimento>,

    /// Dados relativos a reportabilidade fiscal (0..N ocorrências).
    #[serde(rename = "Reportavel", default)]
    pub reportavel: Option<Vec<PaisNascReportavel>>,
}

/// Identificação Fiscal Internacional do Proprietário Beneficiário (`<NIF>`).
#[derive(Debug, Serialize, Deserialize)]
pub struct ProprietariosNif {
    /// Número de identificação fiscal internacional.
    #[serde(rename = "NumeroNIF", default)]
    pub numero_nif: Option<String>,

    /// País de emissão do documento fiscal internacional.
    #[serde(rename = "PaisEmissaoNIF", default)]
    pub pais_emissao_nif: Option<String>,

    /// Texto bruto contido no nó.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// Último nome associado ao país de endereço do proprietário (`<PaisEndereco>`).
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct UltimoNomePaisEndereco {
    /// Sigla identificadora do País.
    #[serde(rename = "Pais", default)]
    pub pais: Option<String>,

    /// Texto bruto contido no nó.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// Local de nascimento cadastrado do Proprietário Beneficiário (`<InfoNascimento>`).
#[derive(Debug, Serialize, Deserialize)]
pub struct LocalNascimento {
    /// Município de nascimento.
    #[serde(rename = "Municipio", default)]
    pub municipio: Option<String>,

    /// Bairro correspondente.
    #[serde(rename = "Bairro", default)]
    pub bairro: Option<String>,

    /// Texto bruto contido no nó.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    /// País de nascimento.
    #[serde(rename = "PaisNasc")]
    pub pais_nasc: PaisDeNascimento,
}

/// País de nascimento estruturado do proprietário (`<PaisNasc>`).
#[derive(Debug, Serialize, Deserialize)]
pub struct PaisDeNascimento {
    /// Sigla identificadora do País.
    #[serde(rename = "Pais", default)]
    pub pais: Option<String>,

    /// Nome antigo associado se aplicável.
    #[serde(rename = "AntigoNomePais", default)]
    pub antigo_nome_pais: Option<String>,

    /// Texto bruto contido no nó.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// Dados de reportabilidade do Proprietário Beneficiário (`<Reportavel>`).
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct PaisNascReportavel {
    /// Sigla correspondente ao País de destinação fiscal.
    #[serde(rename = "Pais", default)]
    pub pais: Option<String>,

    /// Texto bruto contido no nó.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// Bloco contendo dados de competência e datas de movimentação caixa (`<mesCaixa>` / `<caixaAnual>`).
#[derive(Debug, Serialize, Deserialize)]
pub struct MesCaixa {
    /// Período caixa de referência no formato AAAAMM.
    #[serde(rename = "anoMesCaixa", default)]
    pub ano_mes_caixa: Option<String>,

    /// Ano caixa de referência em `evtMovOpFinAnual` (`anoCaixa`).
    #[serde(rename = "anoCaixa", default)]
    pub ano_caixa: Option<String>,

    /// Texto bruto contido no nó.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    /// Movimentações financeiras e contas ativas correspondentes (`movOpFin` ou `movOpFinAnual`).
    #[serde(rename = "movOpFin", alias = "movOpFinAnual", default)]
    pub mov_op_fin: Option<MovOpFin>,
}

impl MesCaixa {
    /// Retorna o período caixa de referência como tipo NaiveDate.
    pub fn get_date_ano_mes_caixa(&self) -> Option<NaiveDate> {
        get_naive_date_from_yyyymm(&self.ano_mes_caixa).or_else(|| {
            self.ano_caixa.as_deref().and_then(|y| {
                let digits = y.remove_non_digits();
                (digits.len() >= 4)
                    .then(|| {
                        NaiveDate::parse_from_str(&format!("{}0101", &digits[..4]), "%Y%m%d").ok()
                    })
                    .flatten()
            })
        })
    }
}

/// Bloco contendo contas e operações de câmbio ativas (`<movOpFin>`).
#[derive(Debug, Serialize, Deserialize)]
pub struct MovOpFin {
    /// Texto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    /// Lista de contas financeiras ativas declaradas.
    #[serde(rename = "Conta", default)]
    pub conta: Option<Vec<Conta>>,

    /// Operações cambiais declaradas se aplicável.
    #[serde(rename = "Cambio", default)]
    pub cambio: Option<Cambio>,
}

/// Dados consolidados de uma Conta Financeira (`<Conta>`).
#[derive(Debug, Serialize, Deserialize)]
pub struct Conta {
    /// Texto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    /// Medidas judiciais e liminares associadas a esta conta se cadastradas.
    #[serde(rename = "MedJudic", default)]
    pub med_judic: Option<Vec<ContaMedJudic>>,

    /// Informações e detalhamento técnico de saldos da conta.
    #[serde(rename = "infoConta")]
    pub info_conta: InfoConta,
}

/// Medidas Judiciais e Liminares vinculadas à conta financeira (`<MedJudic>`).
#[derive(Debug, Serialize, Deserialize)]
pub struct ContaMedJudic {
    /// Número identificador único do processo judicial correspondente.
    #[serde(rename = "NumProcJud", default)]
    pub num_proc_jud: Option<String>,

    /// Vara judicial associada.
    #[serde(rename = "Vara", default)]
    pub vara: Option<String>,

    /// Seção judiciária associada.
    #[serde(rename = "SecJud", default)]
    pub sec_jud: Option<String>,

    /// Subseção judiciária associada se aplicável.
    #[serde(rename = "SubSecJud", default)]
    pub sub_sec_jud: Option<String>,

    /// Data em que ocorreu a concessão da liminar.
    #[serde(rename = "dtConcessao", default)]
    pub dt_concessao: Option<String>,

    /// Data em que ocorreu a cassação da liminar.
    #[serde(rename = "dtCassacao", default)]
    pub dt_cassacao: Option<String>,

    /// Texto bruto contido no nó.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// Detalhamento técnico, titularidade e saldos ativos da conta (`<infoConta>`).
#[derive(Debug, Serialize, Deserialize)]
pub struct InfoConta {
    /// Tipo de conta financeira declarada (corrente, poupança, etc.).
    #[serde(rename = "tpConta", default)]
    pub tp_conta: Option<String>,

    /// Subtipo cadastrado para a conta.
    #[serde(rename = "subTpConta", default)]
    pub sub_tp_conta: Option<String>,

    /// Tipo de numeração adotada.
    #[serde(rename = "tpNumConta", default)]
    pub tp_num_conta: Option<String>,

    /// Número identificador único da conta financeira.
    #[serde(rename = "numConta", default)]
    pub num_conta: Option<String>,

    /// Tipo de relação cadastrada entre o declarado titular e a conta.
    #[serde(rename = "tpRelacaoDeclarado", default)]
    pub tp_relacao_declarado: Option<String>,

    /// Sigla monetária padrão utilizada na conta (moeda corrente).
    #[serde(rename = "moeda", default)]
    pub moeda: Option<String>,

    /// Número total de titulares ativos vinculados à conta.
    #[serde(rename = "NoTitulares", default)]
    pub no_titulares: Option<String>,

    /// Data em que ocorreu a abertura da conta financeira (`dtAberturaConta`).
    #[serde(rename = "dtAberturaConta", default)]
    pub dt_abertura_conta: Option<String>,

    /// Forma de abertura da conta: 1 - Presencial; 2 - Virtual (`formaAberturaConta`).
    #[serde(rename = "formaAberturaConta", default)]
    pub forma_abertura_conta: Option<String>,

    /// Celular utilizado na abertura da conta (`celularAberturaConta`).
    #[serde(rename = "celularAberturaConta", default)]
    pub celular_abertura_conta: Option<String>,

    /// Endereço MAC utilizado na abertura da conta (`MACAddressAberturaConta`).
    #[serde(rename = "MACAddressAberturaConta", default)]
    pub mac_address_abertura_conta: Option<String>,

    /// E-mail de confirmação da conta (`emailConfirmacaoConta` - v1.6.0, ADE Cofis nº 17/2026).
    #[serde(rename = "emailConfirmacaoConta", default)]
    pub email_confirmacao_conta: Option<String>,

    /// Data em que ocorreu o encerramento da conta financeira se aplicável.
    #[serde(rename = "dtEncerramentoConta", default)]
    pub dt_encerramento_conta: Option<String>,

    /// Indicador de inatividade da conta no período caixa.
    #[serde(rename = "IndInatividade", default)]
    pub ind_inatividade: Option<String>,

    /// Indicador de falta de documentação do titular se cadastrado.
    #[serde(rename = "IndNDoc", default)]
    pub ind_ndoc: Option<String>,

    /// Texto bruto contido no nó.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    /// Detalhes de reportabilidade cadastrados vinculados à conta.
    #[serde(rename = "Reportavel", default)]
    pub reportavel: Option<Vec<InfoContaReportavel>>,

    /// Informações cadastrais de intermediários se cadastrados.
    #[serde(rename = "Intermediario", default)]
    pub intermediario: Option<Intermediario>,

    /// Detalhes de fundos de investimento se vinculados à conta.
    #[serde(rename = "Fundo", default)]
    pub fundo: Option<Fundo>,

    /// Balanços, saldos consolidados e créditos/débitos da conta.
    #[serde(rename = "BalancoConta", default)]
    pub balanco_conta: Option<BalancoConta>,

    /// Lista de pagamentos acumulados faturados e retidos.
    #[serde(rename = "PgtosAcum", default)]
    pub pgtos_acum: Vec<PgtosAcum>,
}

impl InfoConta {
    /// Consolida todos os países declarados de reportabilidade fiscal associados a esta conta.
    fn get_paises(&self) -> Option<String> {
        self.reportavel.as_ref().and_then(|infos| {
            // Rust Idiomático: Utiliza filter_map para coletar de forma segura apenas os valores None
            // e .then() para realizar a junção por vírgula somente se a coleção resultante não for vazia.
            let paises: Vec<String> = infos.iter().filter_map(|info| info.pais.clone()).collect();
            (!paises.is_empty()).then(|| paises.join(", "))
        })
    }
}

/// Detalhes cadastrais de reportabilidade fiscal internacional vinculada à conta (`<Reportavel>`).
#[derive(Debug, Serialize, Deserialize)]
pub struct InfoContaReportavel {
    /// Sigla do País de destinação tributária.
    #[serde(rename = "Pais", default)]
    pub pais: Option<String>,

    /// Texto bruto contido no nó.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// Dados cadastrais de Intermediários financeiros envolvidos (`<Intermediario>`).
#[derive(Debug, Serialize, Deserialize)]
pub struct Intermediario {
    /// Código global identificador internacional GIIN.
    #[serde(rename = "GIIN", default)]
    pub giin: Option<String>,

    /// Tipo de número de identificação cadastral (tpNI).
    #[serde(rename = "tpNI", default)]
    pub tp_ni: Option<String>,

    /// Número identificador fiscal cadastrado do intermediário.
    #[serde(rename = "NIIntermediario", default)]
    pub niintermediario: Option<String>,

    /// Texto bruto contido no nó.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// Detalhes relativos a Fundos de Investimento vinculados à conta (`<Fundo>`).
#[derive(Debug, Serialize, Deserialize)]
pub struct Fundo {
    /// Código global identificador internacional GIIN do Fundo.
    #[serde(rename = "GIIN", default)]
    pub giin: Option<String>,

    /// CNPJ correspondente ao Fundo de Investimento.
    #[serde(rename = "CNPJ", default)]
    pub cnpj: Option<String>,

    /// Texto bruto contido no nó.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// Balanços consolidadores de saldos mensais e finais da conta (`<BalancoConta>`).
#[derive(Debug, Serialize, Deserialize)]
pub struct BalancoConta {
    /// Acumulado monetário total de créditos realizados no período caixa.
    #[serde(rename = "totCreditos", default)]
    pub tot_creditos: Option<String>,

    /// Acumulado monetário total de débitos realizados no período caixa.
    #[serde(rename = "totDebitos", default)]
    pub tot_debitos: Option<String>,

    /// Acumulado monetário de transações internas de mesma titularidade (Créditos).
    #[serde(rename = "totCreditosMesmaTitularidade", default)]
    pub tot_creditos_mesma_titularidade: Option<String>,

    /// Acumulado monetário de transações internas de mesma titularidade (Débitos).
    #[serde(rename = "totDebitosMesmaTitularidade", default)]
    pub tot_debitos_mesma_titularidade: Option<String>,

    /// Saldo monetário líquido apurado no último dia útil do período caixa.
    #[serde(rename = "vlrUltDia", default)]
    pub vlr_ult_dia: Option<String>,

    /// Texto bruto contido no nó.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// Dados consolidados de pagamentos mensais ou anuais acumulados (`<PgtosAcum>`).
#[derive(Debug, Serialize, Deserialize)]
pub struct PgtosAcum {
    /// Lista contendo os tipos de pagamentos acumulados faturados.
    #[serde(rename = "tpPgto", default)]
    pub tp_pgto: Vec<String>,

    /// Acumulado monetário total correspondente a estes pagamentos.
    #[serde(rename = "totPgtosAcum", default)]
    pub tot_pgtos_acum: Option<String>,

    /// Texto bruto contido no nó.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// Bloco contendo detalhes relativos a operações de câmbio efetuadas (`<Cambio>`).
#[derive(Debug, Serialize, Deserialize)]
pub struct Cambio {
    /// Acumulado monetário total correspondente a compras de divisas cambiais.
    #[serde(rename = "totCompras", default)]
    pub tot_compras: Option<String>,

    /// Acumulado monetário total correspondente a vendas de divisas cambiais.
    #[serde(rename = "totVendas", default)]
    pub tot_vendas: Option<String>,

    /// Acumulado monetário total correspondente a transferências internacionais.
    #[serde(rename = "totTransferencias", default)]
    pub tot_transferencias: Option<String>,

    /// Texto bruto contido no nó.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    /// Lista de medidas judiciais vinculadas ao câmbio se aplicável.
    #[serde(rename = "MedJudic", default)]
    pub med_judic: Option<Vec<CambioMedJudic>>,
}

/// Medidas Judiciais e Liminares vinculadas a operações de Câmbio (`<MedJudic>`).
#[derive(Debug, Serialize, Deserialize)]
pub struct CambioMedJudic {
    /// Número identificador único do processo judicial.
    #[serde(rename = "NumProcJud", default)]
    pub num_proc_jud: Option<String>,

    /// Vara judicial associada.
    #[serde(rename = "Vara", default)]
    pub vara: Option<String>,

    /// Seção judiciária associada.
    #[serde(rename = "SecJud", default)]
    pub sec_jud: Option<String>,

    /// Subseção judiciária associada se aplicável.
    #[serde(rename = "SubSecJud", default)]
    pub sub_sec_jud: Option<String>,

    /// Data correspondente à concessão da liminar.
    #[serde(rename = "dtConcessao", default)]
    pub dt_concessao: Option<String>,

    /// Data correspondente à cassação da liminar.
    #[serde(rename = "dtCassacao", default)]
    pub dt_cassacao: Option<String>,

    /// Texto bruto contido no nó.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

#[cfg(test)]
mod lib_functions {
    use super::*;
    use crate::XmlParserResult;
    use chrono::NaiveDate;

    #[test]
    /// `cargo test -- --show-output get_struct_field_names`
    #[allow(dead_code)]
    fn get_struct_field_names() -> XmlParserResult<()> {
        my_macro! {
            pub struct Test {
                pub id: Option<String>,
                pub cnpj_do_declarante: Option<String>,
                pub ni_do_declarado: Option<String>,
                pub nome_declarado: Option<String>,
                pub ano_mes_caixa: Option<NaiveDate>,
                pub num_conta: Option<String>,
                pub tot_creditos: f64,
                pub tot_debitos: f64,
            }
        }

        let field_names = Test::get_field_names();

        println!("field_names: {field_names:?}");

        assert_eq!(
            field_names,
            [
                "id",
                "cnpj_do_declarante",
                "ni_do_declarado",
                "nome_declarado",
                "ano_mes_caixa",
                "num_conta",
                "tot_creditos",
                "tot_debitos"
            ]
        );

        Ok(())
    }

    #[test]
    fn test_efinanceira_lote_assincrono_v160() -> XmlParserResult<()> {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <eFinanceira xmlns="http://www.eFinanceira.gov.br/schemas/envioLoteEventosAssincrono/v1_0_0">
            <loteEventosAssincrono>
                <cnpjDeclarante>12345678000100</cnpjDeclarante>
                <eventos>
                    <evento id="ID_LOTE_2026_01">
                        <eFinanceira xmlns="http://www.eFinanceira.gov.br/schemas/evtMovOpFin/v1_6_0">
                            <evtMovOpFin id="ID_EVENT_2026_01">
                                <ideEvento><indRetificacao>1</indRetificacao><tpAmb>1</tpAmb></ideEvento>
                                <ideDeclarante><cnpjDeclarante>12345678000100</cnpjDeclarante></ideDeclarante>
                                <ideDeclarado>
                                    <tpNI>1</tpNI>
                                    <NIDeclarado>00011122233</NIDeclarado>
                                    <inDeclaracaoPropriaCRS>CRS901</inDeclaracaoPropriaCRS>
                                    <NomeDeclarado>CLIENTE EXEMPLO PF</NomeDeclarado>
                                    <Endereco>
                                        <tpEndereco>1</tpEndereco>
                                        <EnderecoEstrutura>
                                            <Logradouro>AVENIDA PAULISTA</Logradouro>
                                            <Numero>1000</Numero>
                                            <CEP>01310100</CEP>
                                            <Municipio>SAO PAULO</Municipio>
                                            <UF>SP</UF>
                                        </EnderecoEstrutura>
                                        <Pais>BR</Pais>
                                    </Endereco>
                                </ideDeclarado>
                                <mesCaixa>
                                    <anoMesCaixa>202601</anoMesCaixa>
                                    <movOpFin>
                                        <Conta>
                                            <infoConta>
                                                <Reportavel><Pais>BR</Pais></Reportavel>
                                                <tpConta>1</tpConta>
                                                <subTpConta>001</subTpConta>
                                                <numConta>12345678</numConta>
                                                <dtAberturaConta>2020-05-15</dtAberturaConta>
                                                <formaAberturaConta>2</formaAberturaConta>
                                                <celularAberturaConta>+5511999999999</celularAberturaConta>
                                                <MACAddressAberturaConta>00:1A:2B:3C:4D:5E</MACAddressAberturaConta>
                                                <emailConfirmacaoConta>cli@example.com</emailConfirmacaoConta>
                                                <BalancoConta>
                                                    <totCreditos>16500,50</totCreditos>
                                                    <totDebitos>200.25</totDebitos>
                                                </BalancoConta>
                                            </infoConta>
                                        </Conta>
                                    </movOpFin>
                                </mesCaixa>
                            </evtMovOpFin>
                        </eFinanceira>
                    </evento>
                </eventos>
            </loteEventosAssincrono>
        </eFinanceira>"#;

        let ef: EFinanceira = quick_xml::de::from_str(xml)?;
        let infos = ef.get_infos();
        assert_eq!(infos.len(), 1);
        assert_eq!(infos[0].id.as_deref(), Some("ID_EVENT_2026_01"));
        assert_eq!(infos[0].sub_tp_conta.as_deref(), Some("001"));
        assert_eq!(
            infos[0].dt_abertura_conta,
            NaiveDate::from_ymd_opt(2020, 5, 15)
        );
        assert_eq!(infos[0].forma_abertura_conta.as_deref(), Some("2"));
        assert_eq!(infos[0].tot_creditos, Some(16500.50));
        assert_eq!(infos[0].tot_debitos, Some(200.25));
        Ok(())
    }

    #[test]
    /// `cargo test -- --show-output struct_iterator`
    fn struct_iterator() -> XmlParserResult<()> {
        let info_a = InfoEFinanceira {
            id: Some("identificação 01".to_string()),
            cnpj_do_declarante: Some("cnpj 01".to_string()),
            ni_do_declarado: Some("ni 01".to_string()),
            nome_declarado: Some("nome 01".to_string()),
            ano_mes_caixa: NaiveDate::from_ymd_opt(2015, 3, 14),
            num_conta: Some("conta 01".to_string()),
            pais_reportado: Some("país 01".to_string()),
            tot_creditos: Some(12.45),
            tot_debitos: Some(0.45),
            ..Default::default()
        };

        let info_b = InfoEFinanceira {
            id: Some("identificação 02".to_string()),
            cnpj_do_declarante: Some("cnpj blabla 02".to_string()),
            ni_do_declarado: Some("ni 02".to_string()),
            nome_declarado: Some("nome 02".to_string()),
            ano_mes_caixa: NaiveDate::from_ymd_opt(2015, 3, 15),
            num_conta: Some("conta 02".to_string()),
            pais_reportado: Some("país 02".to_string()),
            tot_creditos: Some(5.45),
            tot_debitos: None,
            ..Default::default()
        };

        let info_c = InfoEFinanceira {
            id: Some("id blablá".to_string()),
            cnpj_do_declarante: Some("cnpj 03".to_string()),
            ni_do_declarado: Some("ni foo 03".to_string()),
            nome_declarado: Some("nome 03".to_string()),
            ano_mes_caixa: NaiveDate::from_ymd_opt(2015, 3, 16),
            num_conta: Some("conta 03".to_string()),
            pais_reportado: Some("país 03".to_string()),
            tot_creditos: None,
            tot_debitos: Some(-327.4056),
            ..Default::default()
        };

        let infos = [info_a, info_b, info_c];

        infos.iter().enumerate().for_each(|(index, info)| {
            println!(
                "index: {index} ; info: {info:?}, lengths: {:?}\n",
                info.get_lengths()
            );
        });

        Ok(())
    }
}

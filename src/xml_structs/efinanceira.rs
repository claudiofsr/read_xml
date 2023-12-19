/**
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
read_xml -s evtMovOpFin-v1_2_1.xml > evtMovOpFin-v1_2_1.rs
*/

use std::ops::Deref;
use chrono::NaiveDate;
use claudiofsr_lib::StrExtension;
use rust_xlsxwriter::serialize_chrono_option_naive_to_excel;
use serde::{Deserialize, Serialize};
use struct_iterable::Iterable;

use crate::{
    StructExtension,
    excel::InfoExtension,
    xml_structs::assinaturas::Signature,
    get_naive_date_from_yyyymm, 
    Information,
};

// https://stackoverflow.com/questions/29986057/is-there-a-way-to-get-the-field-names-of-a-struct-in-a-macro
// https://danielkeep.github.io/tlborm/book/README.html (The Little Book of Rust Macros)
#[allow(unused_macros)]
macro_rules! my_macro {
    (pub struct $name:ident {
        $(pub $field_name:ident: $field_type:ty,)*
    }) => {
        pub struct $name {
            pub $($field_name: $field_type,)*
        }

        impl $name {
            // This is purely an example—not a good one.
            fn get_field_names() -> Vec<&'static str> {
                vec![$(stringify!($field_name)),*]
            }
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, Iterable)]
pub struct InfoEFinanceira {
    #[serde(rename = "Identificação da Entidade Declarante")]
    pub id: String,
    #[serde(rename = "CNPJ da Entidade Declarante")]
    pub cnpj_do_declarante: String,
    #[serde(rename = "Identificação do declarado")]
    pub ni_do_declarado: String,
    #[serde(rename = "Nome do declarado")]
    pub nome_declarado: String,
    #[serde(rename = "Ano/Mês Caixa Reportado", serialize_with = "serialize_chrono_option_naive_to_excel")]
    pub ano_mes_caixa: Option<NaiveDate>,
    #[serde(rename = "Nº da Conta")]
    pub num_conta: String,
    #[serde(rename = "País Reportado")]
    pub pais_reportado: String,
    #[serde(rename = "Balanço da Conta: Total de Créditos")]
    pub tot_creditos: Option<f64>,
    #[serde(rename = "Balanço da Conta: Total de Débitos")]
    pub tot_debitos: Option<f64>,
}

/// <https://doc.rust-lang.org/book/ch10-02-traits.html#default-implementations>
impl InfoExtension for InfoEFinanceira {}

impl InfoEFinanceira {
    pub fn get_lengths(&self) -> Vec<usize> {
        vec![
            self.id.chars_count(),
            self.cnpj_do_declarante.chars_count(),
            self.ni_do_declarado.chars_count(),
            self.nome_declarado.chars_count(),
            self.ano_mes_caixa.map(|date| date.to_string().chars_count()).unwrap_or_default(),
            self.num_conta.chars_count(),
            self.pais_reportado.chars_count(),
            self.tot_creditos.map(|f|  f.to_string().chars_count()).unwrap_or_default(),
            self.tot_debitos.map(|f| f.to_string().chars_count()).unwrap_or_default(),
        ]
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EFinanceira {
    #[serde(rename = "@versao")]
    pub versao: Option<String>,
    #[serde(rename = "@xmlns")]
    pub xmlns: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "evtMovOpFin")]
    pub evt_mov_op_fin: EvtMovOpFin,
    #[serde(rename = "Signature")]
    pub signature: Signature,
}

/// <https://doc.rust-lang.org/book/ch10-02-traits.html#default-implementations>
impl StructExtension for EFinanceira {
    fn get_information(&self, xml_path: &std::path::Path, arguments: &crate::Arguments) -> Information {
        if arguments.verbose {
            println!("eFinanceira xml_path: {xml_path:?}");
            println!("eFinanceira: {self:#?}\n");
        }
        Information::EFinanceira(self.get_infos())
    }
}

impl EFinanceira {
    pub fn get_id(&self) -> String {
        self
            .evt_mov_op_fin
            .id
            .clone()
    }

    pub fn get_cnpj_do_declarante(&self) -> String {
        self
            .evt_mov_op_fin
            .ide_declarante
            .cnpj_declarante
            .trim()
            .format_cnpj()
    }

    pub fn get_ni_do_declarado(&self) -> String {
        self
            .evt_mov_op_fin
            .ide_declarado
            .nideclarado
            .trim()
            .format_cnpj()
    }

    pub fn get_nome_declarado(&self) -> String {
        self
            .evt_mov_op_fin
            .ide_declarado
            .nome_declarado
            .replace_multiple_whitespaces()
    }

    fn get_contas(&self) -> Vec<&Conta> {
        self
            .evt_mov_op_fin
            .mes_caixa
            .mov_op_fin
            .conta
            .as_ref()
            .into_iter()
            .flatten()
            .collect::<Vec<&Conta>>()
    }

    pub fn get_ano_mes(&self) -> Option<NaiveDate> {
        self
            .evt_mov_op_fin
            .mes_caixa
            .get_date_ano_mes_caixa()
    }

    /// Try to Convert String to Float64.
    ///
    /// "12345,67" -> "12345.67"
    fn to_float64<T>(string: T) -> Option<f64>
    where
        T: Deref<Target=str>,
    {
        string
            .replace(',', ".")
            .parse()
            .ok()
    }

    pub fn get_infos(&self) -> Vec<InfoEFinanceira> {

        let mut infos = Vec::new();
        let contas: Vec<&Conta> = self.get_contas();

        for conta in contas {
            let num_conta = conta.info_conta.num_conta.clone();
            let pais_reportado = conta.info_conta.reportavel.iter().map(|info| info.pais.clone()).collect::<Vec<String>>().join(", ");
            let tot_creditos = Self::to_float64(conta.info_conta.balanco_conta.tot_creditos.as_ref());
            let tot_debitos = Self::to_float64(conta.info_conta.balanco_conta.tot_debitos.as_ref());

            let info_efinanceira = InfoEFinanceira {
                id: self.get_id(),
                cnpj_do_declarante: self.get_cnpj_do_declarante(),
                ni_do_declarado: self.get_ni_do_declarado(),
                nome_declarado: self.get_nome_declarado(),
                ano_mes_caixa: self.get_ano_mes(),
                num_conta,
                pais_reportado,
                tot_creditos,
                tot_debitos,
            };

            infos.push(info_efinanceira.clone());
        }

        infos
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EvtMovOpFin {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "ideEvento")]
    pub ide_evento: IdeEvento,
    #[serde(rename = "ideDeclarante")]
    pub ide_declarante: IdeDeclarante,
    #[serde(rename = "ideDeclarado")]
    pub ide_declarado: IdeDeclarado,
    #[serde(rename = "mesCaixa")]
    pub mes_caixa: MesCaixa,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IdeEvento {
    #[serde(rename = "indRetificacao")]
    pub ind_retificacao: String,
    #[serde(rename = "nrRecibo")]
    pub nr_recibo: Option<String>,
    #[serde(rename = "tpAmb")]
    pub tp_amb: String,
    #[serde(rename = "aplicEmi")]
    pub aplic_emi: String,
    #[serde(rename = "verAplic")]
    pub ver_aplic: String,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IdeDeclarante {
    #[serde(rename = "cnpjDeclarante")]
    pub cnpj_declarante: String,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IdeDeclarado {
    #[serde(rename = "tpNI")]
    pub tp_ni: String,
    #[serde(rename = "tpDeclarado")]
    pub tp_declarado: Option<Vec<String>>,
    #[serde(rename = "NIDeclarado")]
    pub nideclarado: String,
    #[serde(rename = "NomeDeclarado")]
    pub nome_declarado: String,
    #[serde(rename = "tpNomeDeclarado")]
    pub tp_nome_declarado: Option<String>,
    #[serde(rename = "DataNasc")]
    pub data_nasc: Option<String>,
    #[serde(rename = "EnderecoLivre")]
    pub endereco_livre: Option<String>,
    #[serde(rename = "tpEndereco")]
    pub tp_endereco: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "NIF")] // Número de Identificação Fiscal
    pub nif: Option<Vec<IdeDeclaradoNif>>,
    #[serde(rename = "NomeOutros")]
    pub nome_outros: Option<NifNomeOutros>,
    #[serde(rename = "InfoNascimento")]
    pub info_nascimento: Option<NomePjInfoNascimento>,
    #[serde(rename = "PaisEndereco")]
    pub pais_endereco: PaisNascPaisEndereco,
    #[serde(rename = "EnderecoOutros")]
    pub endereco_outros: Option<Vec<PaisEnderecoEnderecoOutros>>,
    #[serde(rename = "paisResid")]
    pub pais_resid: Option<Vec<EnderecoPaisResid>>,
    #[serde(rename = "PaisNacionalidade")]
    pub pais_nacionalidade: Option<Vec<PaisResidPaisNacionalidade>>,
    #[serde(rename = "Proprietarios")]
    pub proprietarios: Option<Vec<Proprietarios>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IdeDeclaradoNif {
    #[serde(rename = "NumeroNIF")]
    pub numero_nif: String,
    #[serde(rename = "PaisEmissaoNIF")]
    pub pais_emissao_nif: String,
    #[serde(rename = "tpNIF")]
    pub tp_nif: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NifNomeOutros {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "NomePF")]
    pub nome_pf: NifNomeOutrosNomePf,
    #[serde(rename = "NomePJ")]
    pub nome_pj: NomePj,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NifNomeOutrosNomePf {
    #[serde(rename = "tpNome")]
    pub tp_nome: Option<String>,
    #[serde(rename = "PrecTitulo")]
    pub prec_titulo: String,
    #[serde(rename = "Titulo")]
    pub titulo: Vec<String>,
    #[serde(rename = "IdGeracao")]
    pub id_geracao: Vec<String>,
    #[serde(rename = "Sufixo")]
    pub sufixo: Vec<String>,
    #[serde(rename = "GenSufixo")]
    pub gen_sufixo: String,
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "PrimeiroNome")]
    pub primeiro_nome: NifNomeOutrosNomePfPrimeiroNome,
    #[serde(rename = "MeioNome")]
    pub meio_nome: Vec<NomeOutrosNomePfPrimeiroNomeMeioNome>,
    #[serde(rename = "PrefixoNome")]
    pub prefixo_nome: NomePfPrimeiroNomeMeioNomePrefixoNome,
    #[serde(rename = "UltimoNome")]
    pub ultimo_nome: PrimeiroNomeMeioNomePrefixoNomeUltimoNome,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NifNomeOutrosNomePfPrimeiroNome {
    #[serde(rename = "Tipo")]
    pub tipo: String,
    #[serde(rename = "Nome")]
    pub nome: String,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NomeOutrosNomePfPrimeiroNomeMeioNome {
    #[serde(rename = "Tipo")]
    pub tipo: String,
    #[serde(rename = "Nome")]
    pub nome: String,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NomePfPrimeiroNomeMeioNomePrefixoNome {
    #[serde(rename = "Tipo")]
    pub tipo: String,
    #[serde(rename = "Nome")]
    pub nome: String,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PrimeiroNomeMeioNomePrefixoNomeUltimoNome {
    #[serde(rename = "Tipo")]
    pub tipo: String,
    #[serde(rename = "Nome")]
    pub nome: String,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NomePj {
    #[serde(rename = "tpNome")]
    pub tp_nome: String,
    #[serde(rename = "Nome")]
    pub nome: String,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NomePjInfoNascimento {
    #[serde(rename = "Municipio")]
    pub municipio: String,
    #[serde(rename = "Bairro")]
    pub bairro: String,
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "PaisNasc")]
    pub pais_nasc: NomePjInfoNascimentoPaisNasc,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NomePjInfoNascimentoPaisNasc {
    #[serde(rename = "Pais")]
    pub pais: String,
    #[serde(rename = "AntigoNomePais")]
    pub antigo_nome_pais: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaisNascPaisEndereco {
    #[serde(rename = "Pais")]
    pub pais: String,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaisEnderecoEnderecoOutros {
    #[serde(rename = "tpEndereco")]
    pub tp_endereco: String,
    #[serde(rename = "Pais")]
    pub pais: String,
    #[serde(rename = "EnderecoLivre")]
    pub endereco_livre: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "EnderecoEstrutura")]
    pub endereco_estrutura: Option<PaisEnderecoEnderecoOutrosEnderecoEstrutura>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaisEnderecoEnderecoOutrosEnderecoEstrutura {
    #[serde(rename = "EnderecoLivre")]
    pub endereco_livre: Option<String>,
    #[serde(rename = "CEP")]
    pub cep: String,
    #[serde(rename = "Municipio")]
    pub municipio: String,
    #[serde(rename = "UF")]
    pub uf: String,
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "Endereco")]
    pub endereco: PaisEnderecoEnderecoOutrosEnderecoEstruturaEndereco,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaisEnderecoEnderecoOutrosEnderecoEstruturaEndereco {
    #[serde(rename = "Logradouro")]
    pub logradouro: String,
    #[serde(rename = "Numero")]
    pub numero: String,
    #[serde(rename = "Complemento")]
    pub complemento: String,
    #[serde(rename = "Andar")]
    pub andar: String,
    #[serde(rename = "Bairro")]
    pub bairro: String,
    #[serde(rename = "CaixaPostal")]
    pub caixa_postal: String,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EnderecoPaisResid {
    #[serde(rename = "Pais")]
    pub pais: String,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaisResidPaisNacionalidade {
    #[serde(rename = "Pais")]
    pub pais: String,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Proprietarios {
    #[serde(rename = "tpNI")]
    pub tp_ni: String,
    #[serde(rename = "NIProprietario")]
    pub niproprietario: String,
    #[serde(rename = "tpProprietario")]
    pub tp_proprietario: Option<String>,
    #[serde(rename = "Nome")]
    pub nome: String,
    #[serde(rename = "tpNome")]
    pub tp_nome: Option<String>,
    #[serde(rename = "EnderecoLivre")]
    pub endereco_livre: Option<String>,
    #[serde(rename = "tpEndereco")]
    pub tp_endereco: Option<String>,
    #[serde(rename = "DataNasc")]
    pub data_nasc: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "NIF")]
    pub nif: Option<ProprietariosNif>,
    #[serde(rename = "NomeOutros")]
    pub nome_outros: Option<Vec<NifNomeOutros>>,
    #[serde(rename = "PaisEndereco")]
    pub pais_endereco: UltimoNomePaisEndereco,
    #[serde(rename = "EnderecoOutros")]
    pub endereco_outros: Option<Vec<PaisEnderecoEnderecoOutros>>,
    #[serde(rename = "paisResid")]
    pub pais_resid: Option<Vec<EnderecoPaisResid>>,
    #[serde(rename = "PaisNacionalidade")]
    pub pais_nacionalidade: Option<Vec<PaisResidPaisNacionalidade>>,
    #[serde(rename = "InfoNascimento")]
    pub info_nascimento: Option<PaisNacionalidadeInfoNascimento>,
    #[serde(rename = "Reportavel")]
    pub reportavel: PaisNascReportavel,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProprietariosNif {
    #[serde(rename = "NumeroNIF")]
    pub numero_nif: String,
    #[serde(rename = "PaisEmissaoNIF")]
    pub pais_emissao_nif: String,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UltimoNomePaisEndereco {
    #[serde(rename = "Pais")]
    pub pais: String,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaisNacionalidadeInfoNascimento {
    #[serde(rename = "Municipio")]
    pub municipio: String,
    #[serde(rename = "Bairro")]
    pub bairro: String,
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "PaisNasc")]
    pub pais_nasc: PaisNacionalidadeInfoNascimentoPaisNasc,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaisNacionalidadeInfoNascimentoPaisNasc {
    #[serde(rename = "Pais")]
    pub pais: Option<String>,
    #[serde(rename = "AntigoNomePais")]
    pub antigo_nome_pais: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaisNascReportavel {
    #[serde(rename = "Pais")]
    pub pais: String,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MesCaixa {
    #[serde(rename = "anoMesCaixa")]
    pub ano_mes_caixa: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "movOpFin")]
    pub mov_op_fin: MovOpFin,
}

impl MesCaixa {
    pub fn get_date_ano_mes_caixa(&self) -> Option<NaiveDate> {
        get_naive_date_from_yyyymm(&self.ano_mes_caixa)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MovOpFin {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "Conta")]
    pub conta: Option<Vec<Conta>>,
    #[serde(rename = "Cambio")]
    pub cambio: Option<Cambio>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Conta {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "MedJudic")]
    pub med_judic: Option<Vec<ContaMedJudic>>,
    #[serde(rename = "infoConta")]
    pub info_conta: InfoConta,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContaMedJudic {
    #[serde(rename = "NumProcJud")]
    pub num_proc_jud: String,
    #[serde(rename = "Vara")]
    pub vara: String,
    #[serde(rename = "SecJud")]
    pub sec_jud: String,
    #[serde(rename = "SubSecJud")]
    pub sub_sec_jud: String,
    #[serde(rename = "dtConcessao")]
    pub dt_concessao: String,
    #[serde(rename = "dtCassacao")]
    pub dt_cassacao: String,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InfoConta {
    #[serde(rename = "tpConta")]
    pub tp_conta: String,
    #[serde(rename = "subTpConta")]
    pub sub_tp_conta: String,
    #[serde(rename = "tpNumConta")]
    pub tp_num_conta: String,
    #[serde(rename = "numConta")]
    pub num_conta: String,
    #[serde(rename = "tpRelacaoDeclarado")]
    pub tp_relacao_declarado: String,
    pub moeda: Option<String>,
    #[serde(rename = "NoTitulares")]
    pub no_titulares: String,
    #[serde(rename = "dtEncerramentoConta")]
    pub dt_encerramento_conta: Option<String>,
    #[serde(rename = "IndInatividade")]
    pub ind_inatividade: Option<String>,
    #[serde(rename = "IndNDoc")]
    pub ind_ndoc: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "Reportavel")]
    pub reportavel: Vec<InfoContaReportavel>,
    #[serde(rename = "Intermediario")]
    pub intermediario: Option<Intermediario>,
    #[serde(rename = "Fundo")]
    pub fundo: Option<Fundo>,
    #[serde(rename = "BalancoConta")]
    pub balanco_conta: BalancoConta,
    #[serde(rename = "PgtosAcum")]
    pub pgtos_acum: Vec<PgtosAcum>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InfoContaReportavel {
    #[serde(rename = "Pais")]
    pub pais: String,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Intermediario {
    #[serde(rename = "GIIN")]
    pub giin: String,
    #[serde(rename = "tpNI")]
    pub tp_ni: String,
    #[serde(rename = "NIIntermediario")]
    pub niintermediario: String,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Fundo {
    #[serde(rename = "GIIN")]
    pub giin: String,
    #[serde(rename = "CNPJ")]
    pub cnpj: String,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BalancoConta {
    #[serde(rename = "totCreditos")]
    pub tot_creditos: String,
    #[serde(rename = "totDebitos")]
    pub tot_debitos: String,
    #[serde(rename = "totCreditosMesmaTitularidade")]
    pub tot_creditos_mesma_titularidade: String,
    #[serde(rename = "totDebitosMesmaTitularidade")]
    pub tot_debitos_mesma_titularidade: String,
    #[serde(rename = "vlrUltDia")]
    pub vlr_ult_dia: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PgtosAcum {
    #[serde(rename = "tpPgto")]
    pub tp_pgto: Vec<String>,
    #[serde(rename = "totPgtosAcum")]
    pub tot_pgtos_acum: String,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Cambio {
    #[serde(rename = "totCompras")]
    pub tot_compras: String,
    #[serde(rename = "totVendas")]
    pub tot_vendas: String,
    #[serde(rename = "totTransferencias")]
    pub tot_transferencias: String,
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "MedJudic")]
    pub med_judic: Option<Vec<CambioMedJudic>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CambioMedJudic {
    #[serde(rename = "NumProcJud")]
    pub num_proc_jud: String,
    #[serde(rename = "Vara")]
    pub vara: String,
    #[serde(rename = "SecJud")]
    pub sec_jud: String,
    #[serde(rename = "SubSecJud")]
    pub sub_sec_jud: String,
    #[serde(rename = "dtConcessao")]
    pub dt_concessao: String,
    #[serde(rename = "dtCassacao")]
    pub dt_cassacao: String,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

#[cfg(test)]
mod lib_functions {
    use super::*;
    use crate::MyResult;
    use chrono::NaiveDate;

    // cargo test -- --help
    // cargo test -- --show-output
    // cargo test -- --show-output multiple_values

    #[test]
    /// https://stackoverflow.com/questions/37140768/how-to-get-struct-field-names-in-rust
    ///
    /// `cargo test -- --show-output get_struct_field_names`
    #[allow(dead_code)]
    fn get_struct_field_names() -> MyResult<()> {

        my_macro! {
            pub struct Test {
                pub id: String,
                pub cnpj_do_declarante: String,
                pub ni_do_declarado: String,
                pub nome_declarado: String,
                pub ano_mes_caixa: Option<NaiveDate>,
                pub num_conta: String,
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
    /// https://stackoverflow.com/questions/76429366/rust-unpack-iter-of-structs-to-iter-of-fields
    ///
    /// `cargo test -- --show-output struct_iterator`
    #[allow(dead_code)]
    fn struct_iterator() -> MyResult<()> {

        let info_a = InfoEFinanceira {
            id: "identificação 01".to_string(),
            cnpj_do_declarante: "cnpj 01".to_string(),
            ni_do_declarado: "ni 01".to_string(),
            nome_declarado: "nome 01".to_string(),
            ano_mes_caixa: NaiveDate::from_ymd_opt(2015, 3, 14),
            num_conta: "conta 01".to_string(),
            pais_reportado: "país 01".to_string(),
            tot_creditos: Some(12.45),
            tot_debitos: Some(0.45),
        };

        let info_b = InfoEFinanceira {
            id: "identificação 02".to_string(),
            cnpj_do_declarante: "cnpj blabla 02".to_string(),
            ni_do_declarado: "ni 02".to_string(),
            nome_declarado: "nome 02".to_string(),
            ano_mes_caixa: NaiveDate::from_ymd_opt(2015, 3, 15),
            num_conta: "conta 02".to_string(),
            pais_reportado: "país 02".to_string(),
            tot_creditos: Some(5.45),
            tot_debitos: None,
        };

        let info_c = InfoEFinanceira {
            id: "id blablá".to_string(),
            cnpj_do_declarante: "cnpj 03".to_string(),
            ni_do_declarado: "ni foo 03".to_string(),
            nome_declarado: "nome 03".to_string(),
            ano_mes_caixa: NaiveDate::from_ymd_opt(2015, 3, 16),
            num_conta: "conta 03".to_string(),
            pais_reportado: "país 03".to_string(),
            tot_creditos: None,
            tot_debitos: Some(-327.4056),
        };

        let infos = vec![info_a, info_b, info_c];

        infos
            .iter()
            .enumerate()
            .for_each(|(index, info)| {
                println!("index: {index} ; info: {info:?}, lengths: {:?}\n", info.get_lengths());
            });

        Ok(())
    }
}
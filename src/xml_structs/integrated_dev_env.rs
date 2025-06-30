use chrono::NaiveDate;
use claudiofsr_lib::OptionExtension;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

use crate::{
    get_naive_date_from_yyyy_mm_dd,
    xml_structs::agente::{Agente, AgenteExtension},
};

/// IDE: Integrated Development Environment
///
/// <https://dfe-portal.svrs.rs.gov.br/CTE/ConsultaSchema>
#[derive(Debug, Serialize, Deserialize)]
pub struct Ide {
    #[serde(rename = "cCT")] // Código numérico que compõe a Chave de Acesso.
    pub c_ct: Option<String>,
    #[serde(rename = "cDV")]
    pub c_dv: Option<String>,
    #[serde(rename = "cMunEnv")]
    // Código do Município de envio do CT-e (de onde o documento foi transmitido)
    pub c_mun_env: Option<String>,
    #[serde(rename = "cMunFG")]
    pub c_mun_fg: Option<String>,
    #[serde(rename = "cMunIni")]
    pub c_mun_ini: Option<String>,
    #[serde(rename = "cMunFim")]
    pub c_mun_fim: Option<String>,
    #[serde(rename = "cNF")]
    pub c_nf: Option<String>,
    #[serde(rename = "cUF")] // Código da UF do emitente
    pub c_uf: Option<String>,
    #[serde(rename = "CFOP")] // Código Fiscal de Operações e Prestações
    pub cfop: Option<String>,
    #[serde(rename = "dhCont")]
    pub dh_cont: Option<String>,
    #[serde(rename = "dEmi")] // Data de emissão
    pub d_emi: Option<String>,
    #[serde(rename = "dhEmi")] // Data e hora de emissão
    pub dh_emi: Option<String>,
    #[serde(rename = "dSaiEnt")] // Data de saída
    pub d_sai_ent: Option<String>,
    #[serde(rename = "dhSaiEnt")] // Data e hora de saída
    pub dh_sai_ent: Option<String>,
    #[serde(rename = "finNFe")]
    pub fin_nfe: Option<String>,
    #[serde(rename = "idDest")]
    pub id_dest: Option<String>,
    #[serde(rename = "indIEToma")]
    pub ind_ietoma: Option<String>,
    #[serde(rename = "indFinal")]
    pub ind_final: Option<String>,
    #[serde(rename = "indGlobalizado")]
    pub ind_globalizado: Option<String>,
    #[serde(rename = "indIntermed")]
    pub ind_intermed: Option<String>,
    #[serde(rename = "indPres")]
    pub ind_pres: Option<String>,
    #[serde(rename = "modal")]
    // Modal: Preencher com:01-Rodoviário; 02-Aéreo;03-Aquaviário;04-Ferroviário;05-Dutoviário;06-Multimodal;
    pub modal: Option<String>,
    #[serde(rename = "mod")] // Modelo do documento fiscal ; CT-e código 57
    pub modelo: Option<String>,
    #[serde(rename = "nCT")] // Número do CT-e
    pub num_cte: Option<String>,
    #[serde(rename = "natOp")] // Natureza da Operação
    pub nat_operacao: Option<String>,
    #[serde(rename = "NFref")]
    pub nfref: Option<Vec<NFref>>,
    #[serde(rename = "nNF")] // Número do NF-e
    pub num_nfe: Option<String>,
    #[serde(rename = "procEmi")]
    pub proc_emi: Option<String>,
    #[serde(rename = "retira")]
    pub retira: Option<String>,
    #[serde(rename = "serie")]
    pub serie: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
    pub toma: Option<String>, // Tomador do Serviço

    // 4 tipos de Tomadores
    // Indicador do "papel" do tomador do serviço no CT-e

    // #[serde(alias = "name")]
    // Specify multiple possible names for the same field.
    #[serde(alias = "toma3", alias = "toma4", alias = "toma03", alias = "toma04")]
    pub tomador: Option<Agente>,

    #[serde(rename = "tpAmb")]
    pub tp_amb: Option<String>,
    #[serde(rename = "tpCTe")]
    // Tipo do CT-e: 0 - CT-e Normal; 1 - CT-e de Complemento de Valores; 2 - CT-e de Anulação; 3 - CT-e de Substituição
    pub tp_cte: Option<String>,
    #[serde(rename = "tpEmis")]
    pub tp_emis: Option<String>,
    #[serde(rename = "tpImp")]
    pub tp_imp: Option<String>,
    #[serde(rename = "tpNF")]
    pub tp_nf: Option<String>,
    #[serde(rename = "tpServ")]
    pub tp_serv: Option<String>,
    #[serde(rename = "UFEnv")]
    pub ufenv: Option<String>,
    #[serde(rename = "UFIni")]
    pub ufini: Option<String>,
    #[serde(rename = "UFFim")]
    pub uffim: Option<String>,
    #[serde(rename = "verProc")]
    pub ver_proc: Option<String>,
    #[serde(rename = "xDetRetira")]
    pub x_det_retira: Option<String>,
    #[serde(rename = "xJust")] // Justificativa da entrada em contingência
    pub x_just: Option<String>,
    #[serde(rename = "xMunEnv")]
    // Nome do Município de envio do CT-e (de onde o documento foi transmitido)
    pub x_mun_env: Option<String>,
    #[serde(rename = "xMunIni")] // Nome do Município do início da prestação
    pub x_mun_ini: Option<String>,
    #[serde(rename = "xMunFim")] // Nome do Município do término da prestação
    pub x_mun_fim: Option<String>,
}

impl Ide {
    pub fn get_num_nfe(&self) -> Option<u32> {
        self.num_nfe
            .as_ref()
            .and_then(|numero| numero.trim().parse::<u32>().ok())
    }

    pub fn get_num_cte(&self) -> Option<u32> {
        self.num_cte
            .as_ref()
            .and_then(|numero| numero.trim().parse::<u32>().ok())
    }

    pub fn get_cfop(&self) -> Option<u16> {
        self.cfop
            .as_ref()
            .and_then(|numero| numero.trim().parse::<u16>().ok())
    }

    pub fn get_dt_emissao(&self) -> Option<NaiveDate> {
        [
            get_naive_date_from_yyyy_mm_dd(&self.d_emi),
            get_naive_date_from_yyyy_mm_dd(&self.dh_emi),
        ]
        .into_iter()
        .find_map(|x| x) // find the first Some(String) value
    }

    pub fn get_dt_saida(&self) -> Option<NaiveDate> {
        [
            get_naive_date_from_yyyy_mm_dd(&self.d_sai_ent),
            get_naive_date_from_yyyy_mm_dd(&self.dh_sai_ent),
        ]
        .into_iter()
        .find_map(|x| x) // find the first Some(String) value
    }

    /// Tomador: CNPJ
    pub fn get_toma_cnpj(&self) -> Option<String> {
        self.tomador.get_ext_cnpj()
    }

    /// Tomador: CPF
    pub fn get_toma_cpf(&self) -> Option<String> {
        self.tomador.get_ext_cpf()
    }

    /// Tomador: Nome
    pub fn get_toma_nome(&self) -> Option<String> {
        self.tomador.get_ext_nome()
    }

    /// Tomador: Fantasia
    pub fn get_toma_fantasia(&self) -> Option<String> {
        self.tomador.get_ext_fantasia()
    }

    /// Tomador: Endereço Município
    pub fn get_toma_ender_municipio(&self) -> Option<String> {
        self.tomador.get_ext_municipio()
    }

    /// Tomador: Endereço Estado
    pub fn get_toma_ender_estado(&self) -> Option<String> {
        self.tomador.get_ext_estado()
    }

    /// Tomador: Código do Tomador
    pub fn get_cod_tomador_0(&self) -> Option<u8> {
        self.toma.parse()
    }

    pub fn get_cod_tomador_1(&self) -> Option<u8> {
        self.tomador.get_ext_tomador()
    }

    /**
    Código do Tomador do Serviço:

    0. Remetente;
    1. Expedidor;
    2. Recebedor;
    3. Destinatário;
    4. Terceiro [adicionado em CTe versão 4.00].
    */
    pub fn get_cod_tomador(&self) -> Option<u8> {
        let tomadores: BTreeSet<u8> = [self.get_cod_tomador_0(), self.get_cod_tomador_1()]
            .into_iter()
            .flatten()
            .collect();

        match tomadores.len() {
            0 => None,
            1 => tomadores.first().copied(),
            _ => {
                eprintln!("Error: CTe com múltiplos tomadores!");
                eprintln!("IDE: {self:#?}");
                eprintln!("Tomadores: {tomadores:#?}\n");
                None
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NFref {
    #[serde(rename = "refNFe")]
    pub ref_nfe: Option<String>,
    #[serde(rename = "refECF")]
    pub ref_ecf: Option<RefEcf>,
    #[serde(rename = "refCTe")]
    pub ref_cte: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "refNF")]
    pub ref_nf: Option<RefNf>,
    #[serde(rename = "refNFP")]
    pub ref_nfp: Option<RefNfp>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefNf {
    #[serde(rename = "cUF")]
    pub c_uf: Option<String>,
    #[serde(rename = "AAMM")]
    pub aamm: Option<String>,
    #[serde(rename = "CNPJ")]
    pub cnpj: Option<String>,
    #[serde(rename = "CPF")]
    pub cpf: Option<String>,
    #[serde(rename = "IE")]
    pub ie: Option<String>,
    #[serde(rename = "mod")]
    pub modelo: Option<String>,
    #[serde(rename = "serie")]
    pub serie: Option<String>,
    #[serde(rename = "nNF")]
    pub n_nf: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefNfp {
    #[serde(rename = "cUF")]
    pub c_uf: Option<String>,
    #[serde(rename = "AAMM")]
    pub aamm: Option<String>,
    #[serde(rename = "CNPJ")]
    pub cnpj: Option<String>,
    #[serde(rename = "CPF")]
    pub cpf: Option<String>,
    #[serde(rename = "IE")]
    pub ie: Option<String>,
    #[serde(rename = "mod")]
    pub modelo: Option<String>,
    #[serde(rename = "serie")]
    pub serie: Option<String>,
    #[serde(rename = "nNF")]
    pub n_nf: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefEcf {
    #[serde(rename = "mod")]
    pub modelo: Option<String>,
    #[serde(rename = "nECF")]
    pub n_ecf: Option<String>,
    #[serde(rename = "nCOO")]
    pub n_coo: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

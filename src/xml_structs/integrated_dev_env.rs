use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::{
    get_naive_date_from_yyyy_mm_dd,
    xml_structs::tomador::Tomador,
};

// IDE: Integrated Development Environment

/// <https://dfe-portal.svrs.rs.gov.br/CTE/ConsultaSchema>
#[derive(Debug, Serialize, Deserialize)]
pub struct Ide {
    #[serde(rename = "cCT")] // Código numérico que compõe a Chave de Acesso.
    pub c_ct: Option<String>,
    #[serde(rename = "cDV")]
    pub c_dv: String,
    #[serde(rename = "cMunEnv")] // Código do Município de envio do CT-e (de onde o documento foi transmitido)
    pub c_mun_env: Option<String>,
    #[serde(rename = "cMunFG")]
    pub c_mun_fg: Option<String>,
    #[serde(rename = "cMunIni")]
    pub c_mun_ini: Option<String>,
    #[serde(rename = "cMunFim")]
    pub c_mun_fim: Option<String>,
    #[serde(rename = "cNF")]
    pub c_nf: Option<String>,
    #[serde(rename = "cUF")] // Código da UF do emitente do CT-e.
    pub c_uf: String,
    #[serde(rename = "CFOP")] // Código Fiscal de Operações e Prestações
    pub cfop: Option<String>,
    #[serde(rename = "dhCont")]
    pub dh_cont: Option<String>,
    #[serde(rename = "dhEmi")] // Data e hora de emissão do CT-e
    pub dh_emi: Option<String>,
    #[serde(rename = "dhSaiEnt")]
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
    #[serde(rename = "modal")] // Modal: Preencher com:01-Rodoviário; 02-Aéreo;03-Aquaviário;04-Ferroviário;05-Dutoviário;06-Multimodal;
    pub modal: Option<String>,
    #[serde(rename = "mod")] // Modelo do documento fiscal ; CT-e código 57
    pub modelo: Option<String>,
    #[serde(rename = "nCT")] // Número do CT-e
    pub num_cte: Option<String>,
    #[serde(rename = "natOp")] // Natureza da Operação
    pub nat_operacao: String,
    #[serde(rename = "NFref")]
    pub nfref: Option<Vec<NFref>>,
    #[serde(rename = "nCT")]
    pub n_ct: Option<String>,
    #[serde(rename = "nNF")]
    pub n_nf: Option<String>,
    #[serde(rename = "procEmi")]
    pub proc_emi: Option<String>,
    #[serde(rename = "retira")]
    pub retira: Option<String>,
    #[serde(rename = "serie")]
    pub serie: String,
    #[serde(rename = "$text")]
    pub text: Option<String>,
    pub toma: Option<String>, // Tomador do Serviço
    #[serde(rename = "toma3")]
    pub toma3: Option<Tomador>, // Indicador do "papel" do tomador do serviço no CT-e
    #[serde(rename = "toma4")]
    pub toma4: Option<Tomador>, // Indicador do "papel" do tomador do serviço no CT-e
     #[serde(rename = "tpAmb")]
    pub tp_amb: String,
    #[serde(rename = "tpCTe")] // Tipo do CT-e: 0 - CT-e Normal; 1 - CT-e de Complemento de Valores; 2 - CT-e de Anulação; 3 - CT-e de Substituição
    pub tp_cte: Option<String>,
    #[serde(rename = "tpEmis")]
    pub tp_emis: String,
    #[serde(rename = "tpImp")]
    pub tp_imp: String,
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
    pub ver_proc: String,
    #[serde(rename = "xDetRetira")]
    pub x_det_retira: Option<String>,
    #[serde(rename = "xJust")] // Justificativa da entrada em contingência
    pub x_just: Option<String>,
    #[serde(rename = "xMunEnv")] // Nome do Município de envio do CT-e (de onde o documento foi transmitido)
    pub x_mun_env: Option<String>,
    #[serde(rename = "xMunIni")] // Nome do Município do início da prestação
    pub x_mun_ini: Option<String>,
    #[serde(rename = "xMunFim")] // Nome do Município do término da prestação
    pub x_mun_fim: Option<String>,
}

impl Ide {
    pub fn get_n_nf(&self) -> Option<u32> {
        self
            .n_nf
            .as_ref()
            .and_then(|numero| {
                numero.trim().parse::<u32>().ok()
            })
    }

    pub fn get_n_ct(&self) -> Option<u32> {
        self
            .num_cte
            .as_ref()
            .and_then(|numero| {
                numero.trim().parse::<u32>().ok()
            })
    }

    pub fn get_cfop(&self) -> Option<u16> {
        self
            .cfop
            .as_ref()
            .and_then(|numero| {
                numero.trim().parse::<u16>().ok()
            })
    }

    pub fn get_dh_emi(&self) -> Option<NaiveDate> {
        get_naive_date_from_yyyy_mm_dd(&self.dh_emi)
    }

    pub fn get_dh_sai(&self) -> Option<NaiveDate> {
        get_naive_date_from_yyyy_mm_dd(&self.dh_sai_ent)
    }


    /// Tomador 3: CNPJ
    pub fn get_toma_3_cnpj(&self) -> Option<String> {
        self
            .toma3
            .as_ref()
            .and_then(|tomador| {
                tomador.get_cnpj()
            })
    }

    /// Tomador 4: CNPJ
    pub fn get_toma_4_cnpj(&self) -> Option<String> {
        self
            .toma4
            .as_ref()
            .and_then(|tomador| {
                tomador.get_cnpj()
            })
    }

    /// Tomador: CNPJ
    pub fn get_toma_cnpj(&self) -> Option<String> {
        let tomador_3 = self.get_toma_3_cnpj();
        let tomador_4 = self.get_toma_4_cnpj();

        if tomador_3.is_some() {
            tomador_3
        } else {
            tomador_4
        }
    }


    /// Tomador 3: CPF
    pub fn get_toma_3_cpf(&self) -> Option<String> {
        self
            .toma3
            .as_ref()
            .and_then(|tomador| {
                tomador.get_cpf()
            })
    }

    /// Tomador 4: CPF
    pub fn get_toma_4_cpf(&self) -> Option<String> {
        self
            .toma4
            .as_ref()
            .and_then(|tomador| {
                tomador.get_cpf()
            })
    }

    /// Tomador: CPF
    pub fn get_toma_cpf(&self) -> Option<String> {
        let tomador_3 = self.get_toma_3_cpf();
        let tomador_4 = self.get_toma_4_cpf();

        if tomador_3.is_some() {
            tomador_3
        } else {
            tomador_4
        }
    }

    /// Tomador 3: Nome
    pub fn get_toma_3_nome(&self) -> Option<String> {
        self
            .toma3
            .as_ref()
            .and_then(|tomador| {
                tomador.get_nome()
            })
    }

    /// Tomador 4: Nome
    pub fn get_toma_4_nome(&self) -> Option<String> {
        self
            .toma4
            .as_ref()
            .and_then(|tomador| {
                tomador.get_nome()
            })
    }

    /// Tomador: Nome
    pub fn get_toma_nome(&self) -> Option<String> {
        let tomador_3 = self.get_toma_3_nome();
        let tomador_4 = self.get_toma_4_nome();

        if tomador_3.is_some() {
            tomador_3
        } else {
            tomador_4
        }
    }

   /// Tomador 3: Fantasia
    pub fn get_toma_3_fantasia(&self) -> Option<String> {
        self
            .toma3
            .as_ref()
            .and_then(|tomador| {
                tomador.get_fantasia()
            })
    }

    /// Tomador 4: Fantasia
    pub fn get_toma_4_fantasia(&self) -> Option<String> {
        self
            .toma4
            .as_ref()
            .and_then(|tomador| {
                tomador.get_fantasia()
            })
    }

    /// Tomador: Fantasia
    pub fn get_toma_fantasia(&self) -> Option<String> {
        let tomador_3 = self.get_toma_3_fantasia();
        let tomador_4 = self.get_toma_4_fantasia();

        if tomador_3.is_some() {
            tomador_3
        } else {
            tomador_4
        }
    }

    /// Tomador 3: Endereço Município
    pub fn get_toma_3_ender_municipio(&self) -> Option<String> {
        self
            .toma3
            .as_ref()
            .and_then(|tomador| {
                tomador.get_endereco_municipio()
            })
    }

    /// Tomador 4: Endereço Município
    pub fn get_toma_4_ender_municipio(&self) -> Option<String> {
        self
            .toma4
            .as_ref()
            .and_then(|tomador| {
                tomador.get_endereco_municipio()
            })
    }

    /// Tomador: Endereço Município
    pub fn get_toma_ender_municipio(&self) -> Option<String> {
        let tomador_3 = self.get_toma_3_ender_municipio();
        let tomador_4 = self.get_toma_4_ender_municipio();

        if tomador_3.is_some() {
            tomador_3
        } else {
            tomador_4
        }
    }

    /// Tomador 3: Endereço Município
    pub fn get_toma_3_ender_estado(&self) -> Option<String> {
        self
            .toma3
            .as_ref()
            .and_then(|tomador| {
                tomador.get_endereco_uf()
            })
    }

    /// Tomador 4: Endereço Município
    pub fn get_toma_4_ender_estado(&self) -> Option<String> {
        self
            .toma4
            .as_ref()
            .and_then(|tomador| {
                tomador.get_endereco_uf()
            })
    }

    /// Tomador: Endereço Município
    pub fn get_toma_ender_estado(&self) -> Option<String> {
        let tomador_3 = self.get_toma_3_ender_estado();
        let tomador_4 = self.get_toma_4_ender_estado();

        if tomador_3.is_some() {
            tomador_3
        } else {
            tomador_4
        }
    }

    /**
    Tomador do Serviço:
        0-Remetente;
        1-Expedidor;
        2-Recebedor;
        3-Destinatário
    */
    pub fn get_cod_tomador_0(&self) -> Option<u8> {
        self
            .toma
            .as_ref()
            .and_then(|codigo| {
                codigo.trim().parse::<u8>().ok()
            })
    }

    pub fn get_cod_tomador_3(&self) -> Option<u8> {
        self
            .toma3
            .as_ref()
            .and_then(|tomador| {
                tomador.get_codigo_do_tomador()
            })
    }

    pub fn get_cod_tomador_4(&self) -> Option<u8> {
        self
            .toma4
            .as_ref()
            .and_then(|tomador| {
                tomador.get_codigo_do_tomador()
            })
    }

    pub fn get_cod_tomador(&self) -> Option<u8> {
        let tomador_0 = self.get_cod_tomador_0();
        let tomador_3 = self.get_cod_tomador_3();
        let tomador_4 = self.get_cod_tomador_4();

        if tomador_0.is_some() {
            tomador_0
        } else if tomador_3.is_some() {
            tomador_3
        } else {
            tomador_4
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Toma3 {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "toma")]
    pub toma: String,
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
    pub modelo: String,
    #[serde(rename = "nECF")]
    pub n_ecf: String,
    #[serde(rename = "nCOO")]
    pub n_coo: String,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}
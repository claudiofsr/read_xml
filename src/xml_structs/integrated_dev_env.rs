//! # Identificação de Documentos Fiscais Eletrônicos (IDE)
//!
//! Este módulo gerencia os dados de identificação do documento fiscal (NFe e CTe),
//! mapeando informações como série, número, datas de emissão, modelo, UF de origem e destino,
//! além de dados sobre o tomador de serviços em conhecimentos de transporte.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::{
    get_naive_date_from_yyyy_mm_dd,
    xml_structs::agente::{Agente, AgenteExtension},
};

/// Dados de Identificação do Documento Fiscal Eletrônico (`<ide>`).
///
/// Contém informações gerais sobre a emissão do documento, parâmetros de controle,
/// contingência, ambiente e modalidade de transporte.
///
/// <https://dfe-portal.svrs.rs.gov.br/CTE/ConsultaSchema>
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Ide {
    /// Código numérico aleatório de controle gerado pelo emitente.
    #[serde(rename = "cCT", default)]
    pub c_ct: Option<String>,

    /// Dígito Verificador da Chave de Acesso.
    #[serde(rename = "cDV", default)]
    pub c_dv: Option<String>,

    /// Código do Município de envio do documento (tabela IBGE).
    #[serde(rename = "cMunEnv", default)]
    pub c_mun_env: Option<String>,

    /// Código do Município de ocorrência do fato gerador.
    #[serde(rename = "cMunFG", default)]
    pub c_mun_fg: Option<String>,

    /// Código do Município de início da prestação/operação.
    #[serde(rename = "cMunIni", default)]
    pub c_mun_ini: Option<String>,

    /// Código do Município de término da prestação/operação.
    #[serde(rename = "cMunFim", default)]
    pub c_mun_fim: Option<String>,

    /// Código numérico que compõe a Chave de Acesso para NF-e.
    #[serde(rename = "cNF", default)]
    pub c_nf: Option<String>,

    /// Código da UF do emitente (tabela IBGE).
    #[serde(rename = "cUF", default)]
    pub c_uf: Option<String>,

    /// Código Fiscal de Operações e Prestações principal.
    #[serde(rename = "CFOP", default)]
    pub cfop: Option<String>,

    /// Data e hora de entrada em contingência.
    #[serde(rename = "dhCont", default)]
    pub dh_cont: Option<String>,

    /// Data de emissão do documento fiscal (formato AAAA-MM-DD).
    #[serde(rename = "dEmi", default)]
    pub d_emi: Option<String>,

    /// Data e hora de emissão do documento fiscal.
    #[serde(rename = "dhEmi", default)]
    pub dh_emi: Option<String>,

    /// Data de saída ou de entrada da mercadoria (formato AAAA-MM-DD).
    #[serde(rename = "dSaiEnt", default)]
    pub d_sai_ent: Option<String>,

    /// Data e hora de saída ou de entrada da mercadoria.
    #[serde(rename = "dhSaiEnt", default)]
    pub dh_sai_ent: Option<String>,

    /// Finalidade de emissão da NF-e (Normal, Complementar, Ajuste, Devolução).
    #[serde(rename = "finNFe", default)]
    pub fin_nfe: Option<String>,

    /// Identificador de destino da operação (Interna, Interestadual, Exterior).
    #[serde(rename = "idDest", default)]
    pub id_dest: Option<String>,

    /// Indicador da Inscrição Estadual do Tomador.
    #[serde(rename = "indIEToma", default)]
    pub ind_ietoma: Option<String>,

    /// Indicador de operação com Consumidor Final.
    #[serde(rename = "indFinal", default)]
    pub ind_final: Option<String>,

    /// Indicador de CT-e Globalizado.
    #[serde(rename = "indGlobalizado", default)]
    pub ind_globalizado: Option<String>,

    /// Indicador de intermediador da operação (canal de venda).
    #[serde(rename = "indIntermed", default)]
    pub ind_intermed: Option<String>,

    /// Indicador de presença do comprador no establishment comercial.
    #[serde(rename = "indPres", default)]
    pub ind_pres: Option<String>,

    /// Modalidade de transporte: 01-Rodoviário; 02-Aéreo; 03-Aquaviário; 04-Ferroviário; 05-Dutoviário; 06-Multimodal.
    #[serde(rename = "modal", default)]
    pub modal: Option<String>,

    /// Modelo do documento fiscal (55 para NF-e, 57 para CT-e).
    #[serde(rename = "mod", default)]
    pub modelo: Option<String>,

    /// Número sequencial do CT-e.
    #[serde(rename = "nCT", default)]
    pub num_cte: Option<String>,

    /// Natureza da Operação comercial (venda, compra, devolução, etc.).
    #[serde(rename = "natOp", default)]
    pub nat_operacao: Option<String>,

    /// Lista de documentos fiscais referenciados.
    #[serde(rename = "NFref", default)]
    pub nfref: Option<Vec<NFref>>,

    /// Número sequencial da NF-e.
    #[serde(rename = "nNF", default)]
    pub num_nfe: Option<String>,

    /// Processo de emissão do documento fiscal.
    #[serde(rename = "procEmi", default)]
    pub proc_emi: Option<String>,

    /// Indicador de retirada de mercadoria se aplicável.
    #[serde(rename = "retira", default)]
    pub retira: Option<String>,

    /// Série do documento fiscal.
    #[serde(rename = "serie", default)]
    pub serie: Option<String>,

    /// Conteúdo de texto bruto associado ao nó XML.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    /// Código do papel do tomador do serviço no CT-e (versão simplificada).
    #[serde(rename = "toma", default)]
    pub toma: Option<String>,

    /// Detalhes cadastrais do tomador do serviço de transporte.
    ///
    /// Mapeia aliases variantes como `toma3`, `toma4`, `toma03` ou `toma04`.
    #[serde(
        alias = "toma3",
        alias = "toma4",
        alias = "toma03",
        alias = "toma04",
        default
    )]
    pub tomador: Option<Agente>,

    /// Tipo de Ambiente de processamento: 1 - Produção; 2 - Homologação.
    #[serde(rename = "tpAmb", default)]
    pub tp_amb: Option<String>,

    /// Tipo do CT-e: 0 - Normal; 1 - Complementar; 2 - Anulação; 3 - Substituição.
    #[serde(rename = "tpCTe", default)]
    pub tp_cte: Option<String>,

    /// Tipo de Emissão do documento fiscal (Normal, Contingência, etc.).
    #[serde(rename = "tpEmis", default)]
    pub tp_emis: Option<String>,

    /// Formato de impressão do documento auxiliar (DANFE/DACTE).
    #[serde(rename = "tpImp", default)]
    pub tp_imp: Option<String>,

    /// Tipo do Documento Fiscal (0 - Entrada; 1 - Saída).
    #[serde(rename = "tpNF", default)]
    pub tp_nf: Option<String>,

    /// Tipo de Serviço de transporte contratado.
    #[serde(rename = "tpServ", default)]
    pub tp_serv: Option<String>,

    /// Sigla da UF correspondente ao envio do documento.
    #[serde(rename = "UFEnv", default)]
    pub ufenv: Option<String>,

    /// Sigla da UF de início da prestação.
    #[serde(rename = "UFIni", default)]
    pub ufini: Option<String>,

    /// Sigla da UF de término da prestação.
    #[serde(rename = "UFFim", default)]
    pub uffim: Option<String>,

    /// Versão do aplicativo emissor.
    #[serde(rename = "verProc", default)]
    pub ver_proc: Option<String>,

    /// Detalhamento complementar do local de retirada.
    #[serde(rename = "xDetRetira", default)]
    pub x_det_retira: Option<String>,

    /// Justificativa de entrada em contingência.
    #[serde(rename = "xJust", default)]
    pub x_just: Option<String>,

    /// Nome do Município de envio do documento.
    #[serde(rename = "xMunEnv", default)]
    pub x_mun_env: Option<String>,

    /// Nome do Município do início da prestação.
    #[serde(rename = "xMunIni", default)]
    pub x_mun_ini: Option<String>,

    /// Nome do Município do término da prestação.
    #[serde(rename = "xMunFim", default)]
    pub x_mun_fim: Option<String>,
}

impl Ide {
    /// Tenta extrair o número da NF-e como um valor inteiro de 32 bits de forma otimizada.
    pub fn get_num_nfe(&self) -> Option<u32> {
        self.num_nfe
            .as_deref()
            .and_then(|numero| numero.trim().parse::<u32>().ok())
    }

    /// Tenta extrair o número do CT-e como um valor inteiro de 32 bits de forma otimizada.
    pub fn get_num_cte(&self) -> Option<u32> {
        self.num_cte
            .as_deref()
            .and_then(|numero| numero.trim().parse::<u32>().ok())
    }

    /// Tenta extrair o CFOP como um valor inteiro de 16 bits de forma otimizada.
    pub fn get_cfop(&self) -> Option<u16> {
        self.cfop
            .as_deref()
            .and_then(|numero| numero.trim().parse::<u16>().ok())
    }

    /// Obtém de forma preguiçosa a data de emissão do documento fiscal a partir de `dEmi` ou de `dhEmi`.
    pub fn get_dt_emissao(&self) -> Option<NaiveDate> {
        get_naive_date_from_yyyy_mm_dd(&self.d_emi)
            .or_else(|| get_naive_date_from_yyyy_mm_dd(&self.dh_emi))
    }

    /// Obtém de forma preguiçosa a data de saída ou movimentação física a partir de `dSaiEnt` ou de `dhSaiEnt`.
    pub fn get_dt_saida(&self) -> Option<NaiveDate> {
        get_naive_date_from_yyyy_mm_dd(&self.d_sai_ent)
            .or_else(|| get_naive_date_from_yyyy_mm_dd(&self.dh_sai_ent))
    }

    /// Extrai o CNPJ cadastrado do Tomador do serviço de transporte se presente.
    pub fn get_toma_cnpj(&self) -> Option<String> {
        self.tomador.get_ext_cnpj()
    }

    /// Extrai o CPF cadastrado do Tomador do serviço de transporte se presente.
    pub fn get_toma_cpf(&self) -> Option<String> {
        self.tomador.get_ext_cpf()
    }

    /// Extrai a Razão Social do Tomador do serviço de transporte se presente.
    pub fn get_toma_nome(&self) -> Option<String> {
        self.tomador.get_ext_nome()
    }

    /// Extrai o Nome Fantasia do Tomador do serviço de transporte se presente.
    pub fn get_toma_fantasia(&self) -> Option<String> {
        self.tomador.get_ext_fantasia()
    }

    /// Extrai o Município correspondente ao endereço do Tomador do serviço.
    pub fn get_toma_ender_municipio(&self) -> Option<String> {
        self.tomador.get_ext_municipio()
    }

    /// Extrai o Estado (UF) correspondente ao endereço do Tomador do serviço.
    pub fn get_toma_ender_estado(&self) -> Option<String> {
        self.tomador.get_ext_estado()
    }

    /// Extrai o código numérico bruto do tomador de serviço a partir da tag simplificada `toma`.
    pub fn get_cod_tomador_0(&self) -> Option<u8> {
        self.toma.as_deref().and_then(|s| s.parse().ok())
    }

    /// Extrai o código numérico bruto do tomador a partir do bloco estruturado cadastrado.
    pub fn get_cod_tomador_1(&self) -> Option<u8> {
        self.tomador.get_ext_tomador()
    }

    /// Retorna o código único correspondente ao Tomador do Serviço de Transporte sem alocações na Heap.
    ///
    /// Código do Tomador do Serviço:
    ///
    /// * `0` - Remetente;
    /// * `1` - Expedidor;
    /// * `2` - Recebedor;
    /// * `3` - Destinatário;
    /// * `4` - Terceiro [adicionado em CT-e versão 4.00].
    pub fn get_cod_tomador(&self) -> Option<u8> {
        match (self.get_cod_tomador_0(), self.get_cod_tomador_1()) {
            (None, None) => None,
            (Some(val), None) | (None, Some(val)) => Some(val),
            (Some(v0), Some(v1)) => {
                if v0 == v1 {
                    Some(v0)
                } else {
                    eprintln!("Error: CTe com múltiplos tomadores divergentes!");
                    eprintln!("IDE: {self:#?}");
                    eprintln!("Tomadores: v0={v0}, v1={v1}\n");
                    None
                }
            }
        }
    }
}

/// Referências de documentos fiscais vinculados (`<NFref>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NFref {
    /// Chave de acesso de uma NF-e referenciada.
    #[serde(rename = "refNFe", default)]
    pub ref_nfe: Option<String>,

    /// Informações de cupom fiscal referenciado emitido por ECF.
    #[serde(rename = "refECF", default)]
    pub ref_ecf: Option<RefEcf>,

    /// Chave de acesso de um CT-e referenciado.
    #[serde(rename = "refCTe", default)]
    pub ref_cte: Option<String>,

    /// Conteúdo de texto bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    /// Referência a uma Nota Fiscal de papel de modelo convencional.
    #[serde(rename = "refNF", default)]
    pub ref_nf: Option<RefNf>,

    /// Referência a uma Nota Fiscal de Produtor Rural de papel.
    #[serde(rename = "refNFP", default)]
    pub ref_nfp: Option<RefNfp>,
}

/// Dados de Nota Fiscal de papel modelo convencional referenciada (`<refNF>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RefNf {
    /// Código da UF do emitente (IBGE).
    #[serde(rename = "cUF", default)]
    pub c_uf: Option<String>,

    /// Ano e Mês de emissão da Nota Fiscal referenciada (formato AA/MM).
    #[serde(rename = "AAMM", default)]
    pub aamm: Option<String>,

    /// CNPJ do emitente da Nota Fiscal referenciada.
    #[serde(rename = "CNPJ", default)]
    pub cnpj: Option<String>,

    /// CPF do emitente da Nota Fiscal referenciada se aplicável.
    #[serde(rename = "CPF", default)]
    pub cpf: Option<String>,

    /// Inscrição Estadual do emitente referenciado.
    #[serde(rename = "IE", default)]
    pub ie: Option<String>,

    /// Modelo do documento fiscal convencional (01 ou 02, etc.).
    #[serde(rename = "mod", default)]
    pub modelo: Option<String>,

    /// Série do documento fiscal convencional.
    #[serde(rename = "serie", default)]
    pub serie: Option<String>,

    /// Número do documento fiscal referenciado.
    #[serde(rename = "nNF", default)]
    pub n_nf: Option<String>,

    /// Conteúdo de texto bruto associado ao nó XML.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// Dados de Nota Fiscal de Produtor Rural de papel referenciada (`<refNFP>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RefNfp {
    /// Código da UF do produtor rural emitente.
    #[serde(rename = "cUF", default)]
    pub c_uf: Option<String>,

    /// Ano e Mês de emissão da nota do produtor (AA/MM).
    #[serde(rename = "AAMM", default)]
    pub aamm: Option<String>,

    /// CNPJ do produtor rural se pessoa jurídica.
    #[serde(rename = "CNPJ", default)]
    pub cnpj: Option<String>,

    /// CPF do produtor rural se pessoa física.
    #[serde(rename = "CPF", default)]
    pub cpf: Option<String>,

    /// Inscrição Estadual do produtor rural referenciado.
    #[serde(rename = "IE", default)]
    pub ie: Option<String>,

    /// Modelo do documento fiscal de produtor (04, etc.).
    #[serde(rename = "mod", default)]
    pub modelo: Option<String>,

    /// Série da nota fiscal de produtor.
    #[serde(rename = "serie", default)]
    pub serie: Option<String>,

    /// Número do documento fiscal de produtor referenciado.
    #[serde(rename = "nNF", default)]
    pub n_nf: Option<String>,

    /// Conteúdo de texto bruto associado ao nó XML.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// Informações de Cupom Fiscal de Emissor de Cupom Fiscal referenciado (`<refECF>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RefEcf {
    /// Código correspondente ao modelo do ECF.
    #[serde(rename = "mod", default)]
    pub modelo: Option<String>,

    /// Número de fabricação do equipamento ECF.
    #[serde(rename = "nECF", default)]
    pub n_ecf: Option<String>,

    /// Número sequencial do Contador de Ordem de Operação (COO).
    #[serde(rename = "nCOO", default)]
    pub n_coo: Option<String>,

    /// Conteúdo de texto bruto associado ao nó XML.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

// =========================================================================
// TESTES UNITÁRIOS - COMPORTAMENTO E VARIANTES DE TOMADORES
// =========================================================================
#[cfg(test)]
mod tests_tomador {
    use super::*;

    #[test]
    /// `cargo test -- --show-output test_get_cod_tomador_permutations_via_xml`
    fn test_get_cod_tomador_permutations_via_xml() {
        // Caso 1: Ambos os campos do tomador estão ausentes (Omitido)
        let xml_1 = r#"<ide></ide>"#;
        let ide_1: Ide = quick_xml::de::from_str(xml_1).expect("Falha ao parsear xml_1");
        assert_eq!(ide_1.get_cod_tomador(), None);

        // Caso 2: Somente o tomador simplificado 'toma' está presente (Código 0)
        let xml_2 = r#"<ide><toma>0</toma></ide>"#;
        let ide_2: Ide = quick_xml::de::from_str(xml_2).expect("Falha ao parsear xml_2");
        assert_eq!(ide_2.get_cod_tomador_0(), Some(0));
        assert_eq!(ide_2.get_cod_tomador_1(), None);
        assert_eq!(ide_2.get_cod_tomador(), Some(0));

        // Caso 3: Somente o tomador estruturado sob o alias 'toma3' está presente (Código 3)
        let xml_3 = r#"<ide><toma3><toma>3</toma></toma3></ide>"#;
        let ide_3: Ide = quick_xml::de::from_str(xml_3).expect("Falha ao parsear xml_3");
        assert_eq!(ide_3.get_cod_tomador_0(), None);
        assert_eq!(ide_3.get_cod_tomador_1(), Some(3));
        assert_eq!(ide_3.get_cod_tomador(), Some(3));

        // Caso 4: Somente o tomador estruturado sob o alias 'toma04' está presente (Código 4)
        let xml_4 = r#"<ide><toma04><toma>4</toma></toma04></ide>"#;
        let ide_4: Ide = quick_xml::de::from_str(xml_4).expect("Falha ao parsear xml_4");
        assert_eq!(ide_4.get_cod_tomador_0(), None);
        assert_eq!(ide_4.get_cod_tomador_1(), Some(4));
        assert_eq!(ide_4.get_cod_tomador(), Some(4));

        // Caso 5: Ambos estão presentes e são idênticos (Consistente)
        let xml_5 = r#"<ide><toma>3</toma><toma3><toma>3</toma></toma3></ide>"#;
        let ide_5: Ide = quick_xml::de::from_str(xml_5).expect("Falha ao parsear xml_5");
        assert_eq!(ide_5.get_cod_tomador_0(), Some(3));
        assert_eq!(ide_5.get_cod_tomador_1(), Some(3));
        assert_eq!(ide_5.get_cod_tomador(), Some(3));

        // Caso 6: Ambos estão presentes mas contêm valores divergentes (Inconsistente)
        let xml_6 = r#"<ide><toma>1</toma><toma3><toma>4</toma></toma3></ide>"#;
        let ide_6: Ide = quick_xml::de::from_str(xml_6).expect("Falha ao parsear xml_6");
        assert_eq!(ide_6.get_cod_tomador_0(), Some(1));
        assert_eq!(ide_6.get_cod_tomador_1(), Some(4));
        assert_eq!(ide_6.get_cod_tomador(), None);
    }
}

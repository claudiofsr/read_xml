//! # Módulo de Tributos e Impostos (NF-e/CT-e)
//!
//! Este módulo gerencia o mapeamento e a desserialização de todos os grupos de impostos
//! incidentes sobre itens de produtos e prestações de serviços de transporte (ICMS, IPI, ISSQN, PIS, COFINS, II).
//!
//! Atualizado para conformidade com o ICMS Monofásico sobre Combustíveis (NT 2023.001 / NT 2023.002)
//! e a modelagem de transição da Reforma Tributária (IBS, CBS e IS).

/*
Abaixo, detalho os motivos técnicos pelos quais o as_deref() sobressai em relação ao as_ref()
quando lidamos com Option<String> (ou Option<Vec<T>>):

1. Tipo resultante mais adequado (Option<&str>)
self.cst.as_ref() converte &Option<String> em Option<&String>.
self.cst.as_deref() converte &Option<String> em Option<&str>.

O tipo &str (fatia de string) é a representação padrão do Rust para strings somente leitura.
Ao transformar o valor interno em &str, você trabalha diretamente com a abstração mais leve do
Rust para strings, sem expor ou depender do contêiner String.

Ao utilizar as_deref(), a conversão para &str ocorre antes de entrar no closure.
Isso torna os tipos explícitos para o compilador e evita que a análise de tipos dependa de
coerções implícitas na chamada de métodos do trait FromStr.
*/

use claudiofsr_lib::OptionExtension;
use serde::{Deserialize, Serialize};

/// Bloco consolidador de tributos e impostos incidentes sobre o item (`<imposto>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Imposto {
    /// Detalhamento do ICMS (Imposto sobre Circulação de Mercadorias e Serviços).
    #[serde(rename = "ICMS", default)]
    pub icms: Option<Icms>,

    /// ICMS devido para a UF de término da prestação do serviço de transporte (Partilha).
    #[serde(rename = "ICMSUFFim", default)]
    pub icmsuffim: Option<Icmsuffim>,

    /// ICMS devido para a UF de destino (Emendas Constitucionais de partilha).
    #[serde(rename = "ICMSUFDest", default)]
    pub icmsufdest: Option<Icmsufdest>,

    /// Imposto de Importação (II).
    #[serde(rename = "II", default)]
    pub ii: Option<Ii>,

    /// Imposto sobre Produtos Industrializados (IPI).
    #[serde(rename = "IPI", default)]
    pub ipi: Option<Vec<Ipi>>,

    /// Imposto Sobre Serviços de Qualquer Natureza (ISSQN).
    #[serde(rename = "ISSQN", default)]
    pub issqn: Option<Issqn>,

    /// Programa de Integração Social (PIS).
    #[serde(rename = "PIS", default)]
    pub pis: Option<Pis>,

    /// Contribuição para o PIS/PASEP sobre a Importação (PISST).
    #[serde(rename = "PISST", default)]
    pub pisst: Option<Pisst>,

    /// Contribuição para o Financiamento da Seguridade Social (COFINS).
    #[serde(rename = "COFINS", default)]
    pub cofins: Option<Cofins>,

    /// Contribuição da Seguridade Social sobre a Importação (COFINSST).
    #[serde(rename = "COFINSST", default)]
    pub cofinsst: Option<Cofinsst>,

    /// Informações adicionais de interesse do Fisco.
    #[serde(rename = "infAdFisco", default)]
    pub inf_ad_fisco: Option<String>,

    /// Valor estimado total de tributos incidentes (Lei da Transparência).
    #[serde(rename = "vTotTrib", default)]
    pub v_tot_trib: Option<String>,

    /// Detalhamento do IBS (Imposto sobre Bens e Serviços - IVA Subnacional).
    #[serde(rename = "IBS", default)]
    pub ibs: Option<Ibs>,

    /// Detalhamento da CBS (Contribuição sobre Bens e Serviços - IVA Federal).
    #[serde(rename = "CBS", default)]
    pub cbs: Option<Cbs>,

    /// Detalhamento consolidado do IBS e CBS (IVA Dual).
    #[serde(rename = "IBSCBS", default)]
    pub ibscbs: Option<Ibscbs>,

    /// Detalhamento do IS (Imposto Seletivo - Extrajudicial/Ambiental).
    #[serde(rename = "IS", default)]
    pub is_tributo: Option<ImpostoSeletivo>,
}

impl Imposto {
    /// Obtém o Código de Situação Tributária (CST) do PIS.
    pub fn get_cst_pis(&self) -> Option<u8> {
        self.pis.as_ref().and_then(|p| p.get_pis_cst())
    }

    /// Obtém a alíquota percentual ou em valor do PIS.
    pub fn get_aliq_pis(&self) -> Option<f64> {
        self.pis.as_ref().and_then(|p| p.get_pis_aliquota())
    }

    /// Obtém o valor monetário calculado do PIS.
    pub fn get_v_pis(&self) -> Option<f64> {
        self.pis.as_ref().and_then(|p| p.get_pis_valor())
    }

    /// Obtém o Código de Situação Tributária (CST) do COFINS.
    pub fn get_cst_cofins(&self) -> Option<u8> {
        self.cofins.as_ref().and_then(|c| c.get_cofins_cst())
    }

    /// Obtém a alíquota percentual ou em valor do COFINS.
    pub fn get_aliq_cofins(&self) -> Option<f64> {
        self.cofins.as_ref().and_then(|c| c.get_cofins_aliquota())
    }

    /// Obtém o valor monetário calculado do COFINS.
    pub fn get_v_cofins(&self) -> Option<f64> {
        self.cofins.as_ref().and_then(|c| c.get_cofins_valor())
    }

    /// Obtém o valor monetário correspondente à Base de Cálculo do ICMS.
    pub fn get_v_bc_icms(&self) -> Option<f64> {
        self.icms
            .as_ref()
            .and_then(|imposto_de_circulacao| imposto_de_circulacao.get_icms_valor_base_calc())
    }

    /// Obtém a alíquota percentual do ICMS.
    pub fn get_aliq_icms(&self) -> Option<f64> {
        self.icms
            .as_ref()
            .and_then(|imposto_de_circulacao| imposto_de_circulacao.get_icms_aliquota())
    }

    /// Obtém o valor monetário calculado do ICMS devido.
    pub fn get_v_icms(&self) -> Option<f64> {
        self.icms
            .as_ref()
            .and_then(|imposto_de_circulacao| imposto_de_circulacao.get_icms_valor_tributo())
    }

    /// Obtém o valor monetário calculado do ISS devido.
    pub fn get_v_iss(&self) -> Option<f64> {
        self.issqn
            .as_ref()
            .and_then(|imposto_sobre_servico| imposto_sobre_servico.get_valor_iss())
    }

    /// Obtém o valor monetário calculado do IPI devido (somando todas as ocorrências).
    pub fn get_v_ipi(&self) -> Option<f64> {
        self.ipi.as_ref().and_then(|imposto_sobre_produto| {
            imposto_sobre_produto
                .iter()
                .map(|tributo| tributo.get_valor_ipi())
                .sum()
        })
    }

    /// Recupera de forma segura o CST do IBS.
    pub fn get_cst_ibs(&self) -> Option<u8> {
        self.ibs.as_ref().and_then(|i| i.get_ibs_cst())
    }

    /// Recupera de forma segura a alíquota do IBS.
    pub fn get_aliq_ibs(&self) -> Option<f64> {
        self.ibs.as_ref().and_then(|i| i.get_ibs_aliquota())
    }

    /// Recupera de forma segura o valor calculado do IBS.
    pub fn get_v_ibs(&self) -> Option<f64> {
        self.ibs.as_ref().and_then(|i| i.get_ibs_valor())
    }

    /// Recupera de forma segura o CST da CBS.
    pub fn get_cst_cbs(&self) -> Option<u8> {
        self.cbs.as_ref().and_then(|c| c.get_cbs_cst())
    }

    /// Recupera de forma segura a alíquota da CBS.
    pub fn get_aliq_cbs(&self) -> Option<f64> {
        self.cbs.as_ref().and_then(|c| c.get_cbs_aliquota())
    }

    /// Recupera de forma segura o valor calculado da CBS.
    pub fn get_v_cbs(&self) -> Option<f64> {
        self.cbs.as_ref().and_then(|c| c.get_cbs_valor())
    }

    /// Recupera de forma segura o CST do Imposto Seletivo (IS).
    pub fn get_cst_is(&self) -> Option<u8> {
        self.is_tributo.as_ref().and_then(|is| is.get_is_cst())
    }

    /// Recupera de forma segura o valor calculado do Imposto Seletivo (IS).
    pub fn get_v_is(&self) -> Option<f64> {
        self.is_tributo.as_ref().and_then(|is| is.get_is_valor())
    }
}

/// Bloco unificador das diversas modalidades de ICMS do leiaute (`<ICMS>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Icms {
    /// Texto descritivo bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    /// Tributação de ICMS Integral (CST 00).
    #[serde(rename = "ICMS00", default)]
    pub icms00: Option<Icms00>,

    /// Tributação de ICMS com cobrança por Substituição Tributária (CST 10).
    #[serde(rename = "ICMS10", default)]
    pub icms10: Option<Icms10>,

    /// Tributação de ICMS com redução de Base de Cálculo (CST 20).
    #[serde(rename = "ICMS20", default)]
    pub icms20: Option<Icms20>,

    /// Tributação de ICMS Isento ou não tributado e com cobrança por ST (CST 30).
    #[serde(rename = "ICMS30", default)]
    pub icms30: Option<Icms30>,

    /// Tributação de ICMS Isento, não tributado ou suspenso (CST 40/41/50).
    #[serde(rename = "ICMS40", default)]
    pub icms40: Option<Icms40>,

    /// Tributação de ICMS Isento, não tributado ou suspenso específico do CT-e (CST 40/41/50).
    #[serde(rename = "ICMS45", default)]
    pub icms45: Option<Icms45>,

    /// Tributação de ICMS com diferimento (CST 51).
    #[serde(rename = "ICMS51", default)]
    pub icms51: Option<Icms51>,

    /// Tributação de ICMS cobrado anteriormente por Substituição Tributária (CST 60).
    #[serde(rename = "ICMS60", default)]
    pub icms60: Option<Icms60>,

    /// Tributação de ICMS com redução de Base de Cálculo e cobrança de ST (CST 70).
    #[serde(rename = "ICMS70", default)]
    pub icms70: Option<Icms70>,

    /// Tributação de ICMS Outros (CST 90).
    #[serde(rename = "ICMS90", default)]
    pub icms90: Option<Icms90>,

    /// Tributação de ICMS de partilha entre UFs (Partilha).
    #[serde(rename = "ICMSPart", default)]
    pub icmspart: Option<Icmspart>,

    /// Tributação de ICMS com substituto tributário retido anteriormente.
    #[serde(rename = "ICMSST", default)]
    pub icmsst: Option<Icmsst>,

    /// Tributação de ICMS pelo Simples Nacional com permissão de crédito (CSOSN 101).
    #[serde(rename = "ICMSSN101", default)]
    pub icmssn101: Option<Icmssn101>,

    /// Tributação de ICMS pelo Simples Nacional sem permissão de crédito (CSOSN 102/103/300/400).
    #[serde(rename = "ICMSSN102", default)]
    pub icmssn102: Option<Icmssn102>,

    /// Tributação de ICMS pelo Simples Nacional com permissão de crédito e cobrança por ST (CSOSN 201).
    #[serde(rename = "ICMSSN201", default)]
    pub icmssn201: Option<Icmssn201>,

    /// Tributação de ICMS pelo Simples Nacional sem permissão de crédito e cobrança por ST (CSOSN 202/203).
    #[serde(rename = "ICMSSN202", default)]
    pub icmssn202: Option<Icmssn202>,

    /// Tributação de ICMS pelo Simples Nacional cobrado anteriormente por ST (CSOSN 500).
    #[serde(rename = "ICMSSN500", default)]
    pub icmssn500: Option<Icmssn500>,

    /// Tributação de ICMS Outros pelo Simples Nacional (CSOSN 900).
    #[serde(rename = "ICMSSN900", default)]
    pub icmssn900: Option<Icmssn900>,

    /// Tributação de ICMS devido a outra UF em operações de transporte.
    #[serde(rename = "ICMSOutraUF", default)]
    pub icmsoutra_uf: Option<IcmsOutraUf>,

    /// Tributação de ICMS simplificada pelo Simples Nacional.
    #[serde(rename = "ICMSSN", default)]
    pub icmssn: Option<Icmssn>,

    /// Tributação monofásica própria sobre combustíveis (CST 02).
    #[serde(rename = "ICMS02", default)]
    pub icms02: Option<Icms02>,

    /// Tributação monofásica própria e com responsabilidade por ST (CST 15).
    #[serde(rename = "ICMS15", default)]
    pub icms15: Option<Icms15>,

    /// Regime monofásico diferido de combustíveis (CST 53).
    #[serde(rename = "ICMS53", default)]
    pub icms53: Option<Icms53>,

    /// Tributação monofásica sobre combustíveis cobrada anteriormente (CST 61).
    #[serde(rename = "ICMS61", default)]
    pub icms61: Option<Icms61>,
}

impl Icms {
    /// Obtém o valor monetário da Base de Cálculo do ICMS de forma declarativa.
    pub fn get_icms_valor_base_calc(&self) -> Option<f64> {
        self.icms00
            .as_ref()
            .and_then(|icms| icms.v_bc.parse_opt())
            .or_else(|| self.icms10.as_ref().and_then(|icms| icms.v_bc.parse_opt()))
            .or_else(|| self.icms20.as_ref().and_then(|icms| icms.v_bc.parse_opt()))
    }

    /// Obtém a alíquota percentual do ICMS de forma declarativa.
    pub fn get_icms_aliquota(&self) -> Option<f64> {
        self.icms00
            .as_ref()
            .and_then(|icms| icms.p_icms.parse_opt())
            .or_else(|| {
                self.icms10
                    .as_ref()
                    .and_then(|icms| icms.p_icms.parse_opt())
            })
            .or_else(|| {
                self.icms20
                    .as_ref()
                    .and_then(|icms| icms.p_icms.parse_opt())
            })
    }

    /// Obtém o valor monetário do ICMS de forma declarativa (incluindo layouts monofásicos).
    pub fn get_icms_valor_tributo(&self) -> Option<f64> {
        self.icms00
            .as_ref()
            .and_then(|icms| icms.v_icms.parse_opt())
            .or_else(|| {
                self.icms10
                    .as_ref()
                    .and_then(|icms| icms.v_icms.parse_opt())
            })
            .or_else(|| {
                self.icms20
                    .as_ref()
                    .and_then(|icms| icms.v_icms.parse_opt())
            })
            // Suporte a ICMS Monofásico Próprio (CST 02)
            .or_else(|| {
                self.icms02
                    .as_ref()
                    .and_then(|icms| icms.v_icms_mono.parse_opt())
            })
            // Suporte a ICMS Monofásico ST (CST 15)
            .or_else(|| {
                self.icms15
                    .as_ref()
                    .and_then(|icms| icms.v_icms_mono.parse_opt())
            })
            // Suporte a ICMS Monofásico retido anteriormente (CST 61)
            .or_else(|| {
                self.icms61
                    .as_ref()
                    .and_then(|icms| icms.v_icms_mono_ret.parse_opt())
            })
    }
}

/// ICMS Integral (CST 00).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Icms00 {
    /// Origem da mercadoria.
    #[serde(rename = "orig", default)]
    pub orig: Option<String>,
    /// Código de Situação Tributária (CST 00).
    #[serde(rename = "CST", default)]
    pub cst: Option<String>,
    /// Modalidade de determinação da base de cálculo.
    #[serde(rename = "modBC", default)]
    pub mod_bc: Option<String>,
    /// Valor monetário da Base de Cálculo do ICMS.
    #[serde(rename = "vBC", default)]
    pub v_bc: Option<String>,
    /// Alíquota percentual do ICMS.
    #[serde(rename = "pICMS", default)]
    pub p_icms: Option<String>,
    /// Valor monetário do ICMS.
    #[serde(rename = "vICMS", default)]
    pub v_icms: Option<String>,
    /// Alíquota do Fundo de Combate à Pobreza (FCP).
    #[serde(rename = "pFCP", default)]
    pub p_fcp: Option<String>,
    /// Valor monetário correspondente ao FCP.
    #[serde(rename = "vFCP", default)]
    pub v_fcp: Option<String>,
    /// Texto descritivo bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// ICMS cobrado com Substituição Tributária (CST 10).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Icms10 {
    /// Origem da mercadoria.
    #[serde(rename = "orig", default)]
    pub orig: Option<String>,
    /// Código de Situação Tributária (CST 10).
    #[serde(rename = "CST", default)]
    pub cst: Option<String>,
    /// Modalidade de determinação da base de cálculo.
    #[serde(rename = "modBC", default)]
    pub mod_bc: Option<String>,
    /// Valor da Base de Cálculo.
    #[serde(rename = "vBC", default)]
    pub v_bc: Option<String>,
    /// Alíquota do ICMS.
    #[serde(rename = "pICMS", default)]
    pub p_icms: Option<String>,
    /// Valor monetário do ICMS.
    #[serde(rename = "vICMS", default)]
    pub v_icms: Option<String>,
    /// Base de cálculo do FCP.
    #[serde(rename = "vBCFCP", default)]
    pub v_bcfcp: Option<String>,
    /// Alíquota do FCP.
    #[serde(rename = "pFCP", default)]
    pub p_fcp: Option<String>,
    /// Valor do FCP.
    #[serde(rename = "vFCP", default)]
    pub v_fcp: Option<String>,
    /// Modalidade de determinação da base de cálculo do ICMS ST.
    #[serde(rename = "modBCST", default)]
    pub mod_bcst: Option<String>,
    /// Percentual de margem de valor adicionado do ICMS ST (MVAST).
    #[serde(rename = "pMVAST", default)]
    pub p_mvast: Option<String>,
    /// Percentual de redução de base de cálculo do ICMS ST.
    #[serde(rename = "pRedBCST", default)]
    pub p_red_bcst: Option<String>,
    /// Valor da base de cálculo do ICMS ST.
    #[serde(rename = "vBCST", default)]
    pub v_bcst: Option<String>,
    /// Alíquota do ICMS ST.
    #[serde(rename = "pICMSST", default)]
    pub p_icmsst: Option<String>,
    /// Valor do ICMS ST.
    #[serde(rename = "vICMSST", default)]
    pub v_icmsst: Option<String>,
    /// Base de cálculo do FCP retido por ST.
    #[serde(rename = "vBCFCPST", default)]
    pub v_bcfcpst: Option<String>,
    /// Alíquota do FCP retido por ST.
    #[serde(rename = "pFCPST", default)]
    pub p_fcpst: Option<String>,
    /// Valor do FCP retido por ST.
    #[serde(rename = "vFCPST", default)]
    pub v_fcpst: Option<String>,
    /// Texto descritivo bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// ICMS com redução de Base de Cálculo (CST 20).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Icms20 {
    /// Origem da mercadoria.
    #[serde(rename = "orig", default)]
    pub orig: Option<String>,
    /// Código de Situação Tributária (CST 20).
    #[serde(rename = "CST", default)]
    pub cst: Option<String>,
    /// Modalidade de determinação da base de cálculo.
    #[serde(rename = "modBC", default)]
    pub mod_bc: Option<String>,
    /// Percentual de redução de base de cálculo.
    #[serde(rename = "pRedBC", default)]
    pub p_red_bc: Option<String>,
    /// Valor da Base de Cálculo.
    #[serde(rename = "vBC", default)]
    pub v_bc: Option<String>,
    /// Alíquota do ICMS.
    #[serde(rename = "pICMS", default)]
    pub p_icms: Option<String>,
    /// Valor monetário do ICMS.
    #[serde(rename = "vICMS", default)]
    pub v_icms: Option<String>,
    /// Base de cálculo do FCP.
    #[serde(rename = "vBCFCP", default)]
    pub v_bcfcp: Option<String>,
    /// Alíquota do FCP.
    #[serde(rename = "pFCP", default)]
    pub p_fcp: Option<String>,
    /// Valor do FCP.
    #[serde(rename = "vFCP", default)]
    pub v_fcp: Option<String>,
    /// Valor do ICMS desonerado se aplicável.
    #[serde(rename = "vICMSDeson", default)]
    pub v_icmsdeson: Option<String>,
    /// Motivo da desoneração do ICMS.
    #[serde(rename = "motDesICMS", default)]
    pub mot_des_icms: Option<String>,
    /// Texto descritivo bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// ICMS Isento ou Não Tributado com cobrança de ST (CST 30).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Icms30 {
    /// Origem da mercadoria.
    #[serde(rename = "orig", default)]
    pub orig: Option<String>,
    /// Código de Situação Tributária (CST 30).
    #[serde(rename = "CST", default)]
    pub cst: Option<String>,
    /// Modalidade de determinação da base de cálculo do ICMS ST.
    #[serde(rename = "modBCST", default)]
    pub mod_bcst: Option<String>,
    /// Margem de valor adicionado (MVAST).
    #[serde(rename = "pMVAST", default)]
    pub p_mvast: Option<String>,
    /// Percentual de redução de base de cálculo do ICMS ST.
    #[serde(rename = "pRedBCST", default)]
    pub p_red_bcst: Option<String>,
    /// Base de cálculo do ICMS ST.
    #[serde(rename = "vBCST", default)]
    pub v_bcst: Option<String>,
    /// Alíquota do ICMS ST.
    #[serde(rename = "pICMSST", default)]
    pub p_icmsst: Option<String>,
    /// Valor do ICMS ST.
    #[serde(rename = "vICMSST", default)]
    pub v_icmsst: Option<String>,
    /// Base de cálculo do FCP retido por ST.
    #[serde(rename = "vBCFCPST", default)]
    pub v_bcfcpst: Option<String>,
    /// Alíquota do FCP retido por ST.
    #[serde(rename = "pFCPST", default)]
    pub p_fcpst: Option<String>,
    /// Valor do FCP retido por ST.
    #[serde(rename = "vFCPST", default)]
    pub v_fcpst: Option<String>,
    /// Valor do ICMS desonerado.
    #[serde(rename = "vICMSDeson", default)]
    pub v_icmsdeson: Option<String>,
    /// Motivo da desoneração do ICMS.
    #[serde(rename = "motDesICMS", default)]
    pub mot_des_icms: Option<String>,
    /// Texto descritivo bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// ICMS Isento, Não Tributado ou Suspenso (CST 40/41/50).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Icms40 {
    /// Origem da mercadoria.
    #[serde(rename = "orig", default)]
    pub orig: Option<String>,
    /// Código de Situação Tributária (CST 40/41/50).
    #[serde(rename = "CST", default)]
    pub cst: Option<String>,
    /// Valor do ICMS desonerado.
    #[serde(rename = "vICMSDeson", default)]
    pub v_icmsdeson: Option<String>,
    /// Motivo da desoneração do ICMS.
    #[serde(rename = "motDesICMS", default)]
    pub mot_des_icms: Option<String>,
    /// Texto descritivo bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// ICMS Isento, Não Tributado ou Suspenso específico do CT-e (CST 40/41/50).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Icms45 {
    /// Código de Situação Tributária (CST 40/41/50).
    #[serde(rename = "CST", default)]
    pub cst: Option<String>,

    /// Valor do ICMS desonerado se aplicável.
    #[serde(rename = "vICMSDeson", default)]
    pub v_icmsdeson: Option<String>,

    /// Motivo da desoneração do ICMS.
    #[serde(rename = "motDesICMS", default)]
    pub mot_des_icms: Option<String>,

    /// Texto descritivo bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// ICMS com Diferimento (CST 51).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Icms51 {
    /// Origem da mercadoria.
    #[serde(rename = "orig", default)]
    pub orig: Option<String>,

    /// Código de Situação Tributária (CST 51).
    #[serde(rename = "CST", default)]
    pub cst: Option<String>,

    /// Modalidade de determinação da base de cálculo.
    #[serde(rename = "modBC", default)]
    pub mod_bc: Option<String>,

    /// Percentual de redução de base de cálculo.
    #[serde(rename = "pRedBC", default)]
    pub p_red_bc: Option<String>,

    /// Valor da base de cálculo do ICMS.
    #[serde(rename = "vBC", default)]
    pub v_bc: Option<String>,

    /// Alíquota do ICMS.
    #[serde(rename = "pICMS", default)]
    pub p_icms: Option<String>,

    /// Valor do ICMS da operação.
    #[serde(rename = "vICMSOp", default)]
    pub v_icmsop: Option<String>,

    /// Percentual de diferimento.
    #[serde(rename = "pDiff", alias = "pDif", default)] // Aceita ambas as grafias de leiaute
    pub p_dif: Option<String>,

    /// Valor do ICMS diferido (vICMSDif).
    #[serde(rename = "vICMSDif", default)]
    pub v_icms_dif: Option<String>,

    /// Valor do ICMS.
    #[serde(rename = "vICMS", default)]
    pub v_icms: Option<String>,

    /// Base de cálculo do FCP.
    #[serde(rename = "vBCFCP", default)]
    pub v_bcfcp: Option<String>,

    /// Alíquota do FCP.
    #[serde(rename = "pFCP", default)]
    pub p_fcp: Option<String>,

    /// Valor do FCP.
    #[serde(rename = "vFCP", default)]
    pub v_fcp: Option<String>,

    /// Texto descritivo bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// ICMS cobrado anteriormente por Substituição Tributária (CST 60).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Icms60 {
    /// Origem da mercadoria.
    #[serde(rename = "orig", default)]
    pub orig: Option<String>,
    /// Código de Situação Tributária (CST 60).
    #[serde(rename = "CST", default)]
    pub cst: Option<String>,
    /// Base de cálculo do ICMS ST retido anteriormente.
    #[serde(rename = "vBCSTRet", default)]
    pub v_bcstret: Option<String>,
    /// Alíquota do ICMS ST retido anteriormente.
    #[serde(rename = "pST", default)]
    pub p_st: Option<String>,
    /// Alíquota do ICMS ST retido anteriormente (pICMSSTRet).
    #[serde(rename = "pICMSSTRet", default)]
    pub p_icmsst_ret: Option<String>,
    /// Valor do ICMS substituto.
    #[serde(rename = "vICMSSubstituto", default)]
    pub v_icmssubstituto: Option<String>,
    /// Valor do ICMS ST retido anteriormente.
    #[serde(rename = "vICMSSTRet", default)]
    pub v_icmsstret: Option<String>,
    /// Base de cálculo do FCP retido anteriormente por ST.
    #[serde(rename = "vBCFCPSTRet", default)]
    pub v_bcfcpstret: Option<String>,
    /// Alíquota do FCP retido anteriormente por ST.
    #[serde(rename = "pFCPSTRet", default)]
    pub p_fcpstret: Option<String>,
    /// Valor do FCP retido anteriormente por ST.
    #[serde(rename = "vFCPSTRet", default)]
    pub v_fcpstret: Option<String>,
    /// Percentual de redução de base de cálculo efetiva.
    #[serde(rename = "pRedBCEfet", default)]
    pub p_red_bcefet: Option<String>,
    /// Valor da base de cálculo efetiva.
    #[serde(rename = "vBCEfet", default)]
    pub v_bcefet: Option<String>,
    /// Alíquota do ICMS efetiva.
    #[serde(rename = "pICMSEfet", default)]
    pub p_icmsefet: Option<String>,
    /// Valor do ICMS efetivo.
    #[serde(rename = "vICMSEfet", default)]
    pub v_icmsefet: Option<String>,
    /// Texto descritivo bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// ICMS com redução de Base de Cálculo e cobrança de ST (CST 70).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Icms70 {
    /// Origem da mercadoria.
    #[serde(rename = "orig", default)]
    pub orig: Option<String>,
    /// Código de Situação Tributária (CST 70).
    #[serde(rename = "CST", default)]
    pub cst: Option<String>,
    /// Modalidade de determinação da base de cálculo do ICMS.
    #[serde(rename = "modBC", default)]
    pub mod_bc: Option<String>,
    /// Percentual de redução de base de cálculo do ICMS.
    #[serde(rename = "pRedBC", default)]
    pub p_red_bc: Option<String>,
    /// Base de cálculo do ICMS.
    #[serde(rename = "vBC", default)]
    pub v_bc: Option<String>,
    /// Alíquota do ICMS.
    #[serde(rename = "pICMS", default)]
    pub p_icms: Option<String>,
    /// Valor monetário do ICMS.
    #[serde(rename = "vICMS", default)]
    pub v_icms: Option<String>,
    /// Base de cálculo do FCP.
    #[serde(rename = "vBCFCP", default)]
    pub v_bcfcp: Option<String>,
    /// Alíquota do FCP.
    #[serde(rename = "pFCP", default)]
    pub p_fcp: Option<String>,
    /// Valor do FCP.
    #[serde(rename = "vFCP", default)]
    pub v_fcp: Option<String>,
    /// Modalidade de determinação da base de cálculo do ICMS ST.
    #[serde(rename = "modBCST", default)]
    pub mod_bcst: Option<String>,
    /// Margem de valor adicionado do ICMS ST.
    #[serde(rename = "pMVAST", default)]
    pub p_mvast: Option<String>,
    /// Percentual de redução de base de cálculo do ICMS ST.
    #[serde(rename = "pRedBCST", default)]
    pub p_red_bcst: Option<String>,
    /// Base de cálculo do ICMS ST.
    #[serde(rename = "vBCST", default)]
    pub v_bcst: Option<String>,
    /// Alíquota do ICMS ST.
    #[serde(rename = "pICMSST", default)]
    pub p_icmsst: Option<String>,
    /// Valor do ICMS ST.
    #[serde(rename = "vICMSST", default)]
    pub v_icmsst: Option<String>,
    /// Base de cálculo do FCP retido por ST.
    #[serde(rename = "vBCFCPST", default)]
    pub v_bcfcpst: Option<String>,
    /// Alíquota do FCP retido por ST.
    #[serde(rename = "pFCPST", default)]
    pub p_fcpst: Option<String>,
    /// Valor do FCP retido por ST.
    #[serde(rename = "vFCPST", default)]
    pub v_fcpst: Option<String>,
    /// Valor do ICMS desonerado.
    #[serde(rename = "vICMSDeson", default)]
    pub v_icmsdeson: Option<String>,
    /// Motivo da desoneração do ICMS.
    #[serde(rename = "motDesICMS", default)]
    pub mot_des_icms: Option<String>,
    /// Texto descritivo bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// ICMS Outros (CST 90).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Icms90 {
    /// Origem da mercadoria.
    #[serde(rename = "orig", default)]
    pub orig: Option<String>,
    /// Código de Situação Tributária (CST 90).
    #[serde(rename = "CST", default)]
    pub cst: Option<String>,
    /// Modalidade de determinação da base de cálculo do ICMS.
    #[serde(rename = "modBC", default)]
    pub mod_bc: Option<String>,
    /// Base de cálculo do ICMS.
    #[serde(rename = "vBC", default)]
    pub v_bc: Option<String>,
    /// Percentual de redução de base de cálculo do ICMS.
    #[serde(rename = "pRedBC", default)]
    pub p_red_bc: Option<String>,
    /// Alíquota do ICMS.
    #[serde(rename = "pICMS", default)]
    pub p_icms: Option<String>,
    /// Valor monetário do ICMS.
    #[serde(rename = "vICMS", default)]
    pub v_icms: Option<String>,
    /// Base de cálculo do FCP.
    #[serde(rename = "vBCFCP", default)]
    pub v_bcfcp: Option<String>,
    /// Alíquota do FCP.
    #[serde(rename = "pFCP", default)]
    pub p_fcp: Option<String>,
    /// Valor do FCP.
    #[serde(rename = "vFCP", default)]
    pub v_fcp: Option<String>,
    /// Modalidade de determinação da base de cálculo do ICMS ST.
    #[serde(rename = "modBCST", default)]
    pub mod_bcst: Option<String>,
    /// Margem de valor adicionado do ICMS ST.
    #[serde(rename = "pMVAST", default)]
    pub p_mvast: Option<String>,
    /// Percentual de redução de base de cálculo do ICMS ST.
    #[serde(rename = "pRedBCST", default)]
    pub p_red_bcst: Option<String>,
    /// Base de cálculo do ICMS ST.
    #[serde(rename = "vBCST", default)]
    pub v_bcst: Option<String>,
    /// Alíquota do ICMS ST.
    #[serde(rename = "pICMSST", default)]
    pub p_icmsst: Option<String>,
    /// Valor do ICMS ST.
    #[serde(rename = "vICMSST", default)]
    pub v_icmsst: Option<String>,
    /// Base de cálculo do FCP retido por ST.
    #[serde(rename = "vBCFCPST", default)]
    pub v_bcfcpst: Option<String>,
    /// Alíquota do FCP retido por ST.
    #[serde(rename = "pFCPST", default)]
    pub p_fcpst: Option<String>,
    /// Valor do FCP retido por ST.
    #[serde(rename = "vFCPST", default)]
    pub v_fcpst: Option<String>,
    /// Valor do ICMS desonerado.
    #[serde(rename = "vICMSDeson", default)]
    pub v_icmsdeson: Option<String>,
    /// Motivo da desoneração do ICMS.
    #[serde(rename = "motDesICMS", default)]
    pub mot_des_icms: Option<String>,
    /// Texto descritivo bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// ICMS devido a outra UF em operações de transporte (`<ICMSOutraUF>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IcmsOutraUf {
    /// Texto descritivo bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
    /// Código de Situação Tributária.
    #[serde(rename = "CST", default)]
    pub cst: Option<String>,
    /// Alíquota do ICMS aplicável na outra UF.
    #[serde(rename = "pICMSOutraUF", default)]
    pub p_icmsoutra_uf: Option<String>,
    /// Percentual de redução de base de cálculo na outra UF.
    #[serde(rename = "pRedBCOutraUF", default)]
    pub p_red_bcoutra_uf: Option<String>,
    /// Base de cálculo do ICMS na outra UF.
    #[serde(rename = "vBCOutraUF", default)]
    pub v_bcoutra_uf: Option<String>,
    /// Valor do ICMS devido à outra UF.
    #[serde(rename = "vICMSOutraUF", default)]
    pub v_icmsoutra_uf: Option<String>,
}

/// Mapeamento básico para optantes do Simples Nacional (`<ICMSSN>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Icmssn {
    /// Texto descritivo bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
    /// Código de Situação Tributária / CSOSN.
    #[serde(rename = "CST", default)]
    pub cst: Option<String>,
    /// Indicador de Simples Nacional.
    #[serde(rename = "indSN", default)]
    pub ind_sn: Option<String>,
}

/// ICMS de Partilha entre UFs (`<ICMSPart>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Icmspart {
    /// Origem da mercadoria.
    #[serde(rename = "orig", default)]
    pub orig: Option<String>,
    /// Código de Situação Tributária.
    #[serde(rename = "CST", default)]
    pub cst: Option<String>,
    /// Modalidade de determinação da base de cálculo.
    #[serde(rename = "modBC", default)]
    pub mod_bc: Option<String>,
    /// Base de cálculo do ICMS.
    #[serde(rename = "vBC", default)]
    pub v_bc: Option<String>,
    /// Percentual de redução de base de cálculo.
    #[serde(rename = "pRedBC", default)]
    pub p_red_bc: Option<String>,
    /// Alíquota do ICMS da operação.
    #[serde(rename = "pICMS", default)]
    pub p_icms: Option<String>,
    /// Valor monetário do ICMS.
    #[serde(rename = "vICMS", default)]
    pub v_icms: Option<String>,
    /// Modalidade de determinação da base de cálculo do ICMS ST.
    #[serde(rename = "modBCST", default)]
    pub mod_bcst: Option<String>,
    /// Margem de valor adicionado do ICMS ST.
    #[serde(rename = "pMVAST", default)]
    pub p_mvast: Option<String>,
    /// Percentual de redução de base de cálculo do ICMS ST.
    #[serde(rename = "pRedBCST", default)]
    pub p_red_bcst: Option<String>,
    /// Base de cálculo do ICMS ST.
    #[serde(rename = "vBCST", default)]
    pub v_bcst: Option<String>,
    /// Alíquota do ICMS ST.
    #[serde(rename = "pICMSST", default)]
    pub p_icmsst: Option<String>,
    /// Valor do ICMS ST.
    #[serde(rename = "vICMSST", default)]
    pub v_icmsst: Option<String>,
    /// Alíquota para cálculo do crédito de operação própria.
    #[serde(rename = "pBCOp", default)]
    pub p_bcop: Option<String>,
    /// Sigla da UF do substituto tributário.
    #[serde(rename = "UFST", default)]
    pub ufst: Option<String>,
    /// Texto descritivo bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// ICMS retido anteriormente por substituto tributário (`<ICMSST>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Icmsst {
    /// Origem da mercadoria.
    #[serde(rename = "orig", default)]
    pub orig: Option<String>,
    /// Código de Situação Tributária.
    #[serde(rename = "CST", default)]
    pub cst: Option<String>,
    /// Base de cálculo do ICMS ST retido anteriormente.
    #[serde(rename = "vBCSTRet", default)]
    pub v_bcstret: Option<String>,
    /// Alíquota do ICMS ST retido anteriormente.
    #[serde(rename = "pST", default)]
    pub p_st: Option<String>,
    /// Valor do ICMS substituto.
    #[serde(rename = "vICMSSubstituto", default)]
    pub v_icmssubstituto: Option<String>,
    /// Valor do ICMS ST retido anteriormente.
    #[serde(rename = "vICMSSTRet", default)]
    pub v_icmsstret: Option<String>,
    /// Base de cálculo do FCP retido anteriormente por ST.
    #[serde(rename = "vBCFCPSTRet", default)]
    pub v_bcfcpstret: Option<String>,
    /// Alíquota do FCP retido anteriormente por ST.
    #[serde(rename = "pFCPSTRet", default)]
    pub p_fcpstret: Option<String>,
    /// Valor do FCP retido anteriormente por ST.
    #[serde(rename = "vFCPSTRet", default)]
    pub v_fcpstret: Option<String>,
    /// Base de cálculo do ICMS ST devido à UF de destino.
    #[serde(rename = "vBCSTDest", default)]
    pub v_bcstdest: Option<String>,
    /// Valor do ICMS ST devido à UF de destino.
    #[serde(rename = "vICMSSTDest", default)]
    pub v_icmsstdest: Option<String>,
    /// Percentual de redução de base de cálculo efetiva.
    #[serde(rename = "pRedBCEfet", default)]
    pub p_red_bcefet: Option<String>,
    /// Valor da base de cálculo efetiva.
    #[serde(rename = "vBCEfet", default)]
    pub v_bcefet: Option<String>,
    /// Alíquota do ICMS efetiva.
    #[serde(rename = "pICMSEfet", default)]
    pub p_icmsefet: Option<String>,
    /// Valor do ICMS efetivo.
    #[serde(rename = "vICMSEfet", default)]
    pub v_icmsefet: Option<String>,
    /// Texto descritivo bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// Simples Nacional com permissão de crédito (CSOSN 101).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Icmssn101 {
    /// Origem da mercadoria.
    #[serde(rename = "orig", default)]
    pub orig: Option<String>,
    /// CSOSN 101.
    #[serde(rename = "CSOSN", default)]
    pub csosn: Option<String>,
    /// Alíquota aplicável para cálculo do crédito.
    #[serde(rename = "pCredSN", default)]
    pub p_cred_sn: Option<String>,
    /// Valor monetário correspondente ao crédito do ICMS.
    #[serde(rename = "vCredICMSSN", default)]
    pub v_cred_icmssn: Option<String>,
    /// Texto descritivo bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// Simples Nacional sem permissão de crédito (CSOSN 102/103/300/400).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Icmssn102 {
    /// Origem da mercadoria.
    #[serde(rename = "orig", default)]
    pub orig: Option<String>,
    /// CSOSN correspondente.
    #[serde(rename = "CSOSN", default)]
    pub csosn: Option<String>,
    /// Texto descritivo bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// Simples Nacional com permissão de crédito e cobrança por ST (CSOSN 201).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Icmssn201 {
    /// Origem da mercadoria.
    #[serde(rename = "orig", default)]
    pub orig: Option<String>,
    /// CSOSN 201.
    #[serde(rename = "CSOSN", default)]
    pub csosn: Option<String>,
    /// Modalidade de determinação da base de cálculo do ICMS ST.
    #[serde(rename = "modBCST", default)]
    pub mod_bcst: Option<String>,
    /// Margem de valor adicionado do ICMS ST.
    #[serde(rename = "pMVAST", default)]
    pub p_mvast: Option<String>,
    /// Percentual de redução de base de cálculo do ICMS ST.
    #[serde(rename = "pRedBCST", default)]
    pub p_red_bcst: Option<String>,
    /// Base de cálculo do ICMS ST.
    #[serde(rename = "vBCST", default)]
    pub v_bcst: Option<String>,
    /// Alíquota do ICMS ST.
    #[serde(rename = "pICMSST", default)]
    pub p_icmsst: Option<String>,
    /// Valor do ICMS ST.
    #[serde(rename = "vICMSST", default)]
    pub v_icmsst: Option<String>,
    /// Base de cálculo do FCP retido por ST.
    #[serde(rename = "vBCFCPST", default)]
    pub v_bcfcpst: Option<String>,
    /// Alíquota do FCP retido por ST.
    #[serde(rename = "pFCPST", default)]
    pub p_fcpst: Option<String>,
    /// Valor do FCP retido por ST.
    #[serde(rename = "vFCPST", default)]
    pub v_fcpst: Option<String>,
    /// Alíquota aplicável para cálculo do crédito no Simples Nacional.
    #[serde(rename = "pCredSN", default)]
    pub p_cred_sn: Option<String>,
    /// Valor do crédito de ICMS do Simples Nacional.
    #[serde(rename = "vCredICMSSN", default)]
    pub v_cred_icmssn: Option<String>,
    /// Texto descritivo bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// Simples Nacional sem permissão de crédito e cobrança por ST (CSOSN 202/203).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Icmssn202 {
    /// Origem da mercadoria.
    #[serde(rename = "orig", default)]
    pub orig: Option<String>,
    /// CSOSN 202/203.
    #[serde(rename = "CSOSN", default)]
    pub csosn: Option<String>,
    /// Modalidade de determinação da base de cálculo do ICMS ST.
    #[serde(rename = "modBCST", default)]
    pub mod_bcst: Option<String>,
    /// Margem de valor adicionado do ICMS ST.
    #[serde(rename = "pMVAST", default)]
    pub p_mvast: Option<String>,
    /// Percentual de redução de base de cálculo do ICMS ST.
    #[serde(rename = "pRedBCST", default)]
    pub p_red_bcst: Option<String>,
    /// Base de cálculo do ICMS ST.
    #[serde(rename = "vBCST", default)]
    pub v_bcst: Option<String>,
    /// Alíquota do ICMS ST.
    #[serde(rename = "pICMSST", default)]
    pub p_icmsst: Option<String>,
    /// Valor do ICMS ST.
    #[serde(rename = "vICMSST", default)]
    pub v_icmsst: Option<String>,
    /// Base de cálculo do FCP retido por ST.
    #[serde(rename = "vBCFCPST", default)]
    pub v_bcfcpst: Option<String>,
    /// Alíquota do FCP retido por ST.
    #[serde(rename = "pFCPST", default)]
    pub p_fcpst: Option<String>,
    /// Valor do FCP retido por ST.
    #[serde(rename = "vFCPST", default)]
    pub v_fcpst: Option<String>,
    /// Texto descritivo bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// Simples Nacional com ICMS cobrado anteriormente por ST (CSOSN 500).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Icmssn500 {
    /// Origem da mercadoria.
    #[serde(rename = "orig", default)]
    pub orig: Option<String>,
    /// CSOSN 500.
    #[serde(rename = "CSOSN", default)]
    pub csosn: Option<String>,
    /// Base de cálculo do ICMS ST retido anteriormente.
    #[serde(rename = "vBCSTRet", default)]
    pub v_bcstret: Option<String>,
    /// Alíquota do ICMS ST retido anteriormente.
    #[serde(rename = "pST", default)]
    pub p_st: Option<String>,
    /// Alíquota do ICMS ST retido anteriormente (pICMSSTRet).
    #[serde(rename = "pICMSSTRet", default)]
    pub p_icmsst_ret: Option<String>,
    /// Valor do ICMS substituto.
    #[serde(rename = "vICMSSubstituto", default)]
    pub v_icmssubstituto: Option<String>,
    /// Valor do ICMS ST retido anteriormente.
    #[serde(rename = "vICMSSTRet", default)]
    pub v_icmsstret: Option<String>,
    /// Base de cálculo do FCP retido anteriormente por ST.
    #[serde(rename = "vBCFCPSTRet", default)]
    pub v_bcfcpstret: Option<String>,
    /// Alíquota do FCP retido anteriormente por ST.
    #[serde(rename = "pFCPSTRet", default)]
    pub p_fcpstret: Option<String>,
    /// Valor do FCP retido anteriormente por ST.
    #[serde(rename = "vFCPSTRet", default)]
    pub v_fcpstret: Option<String>,
    /// Percentual de redução de base de cálculo efetiva.
    #[serde(rename = "pRedBCEfet", default)]
    pub p_red_bcefet: Option<String>,
    /// Valor da base de cálculo efetiva.
    #[serde(rename = "vBCEfet", default)]
    pub v_bcefet: Option<String>,
    /// Alíquota do ICMS efetiva.
    #[serde(rename = "pICMSEfet", default)]
    pub p_icmsefet: Option<String>,
    /// Valor do ICMS efetivo.
    #[serde(rename = "vICMSEfet", default)]
    pub v_icmsefet: Option<String>,
    /// Texto descritivo bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// Simples Nacional Outros (CSOSN 900).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Icmssn900 {
    /// Origem da mercadoria.
    #[serde(rename = "orig", default)]
    pub orig: Option<String>,
    /// CSOSN 900.
    #[serde(rename = "CSOSN", default)]
    pub csosn: Option<String>,
    /// Modalidade de determinação da base de cálculo do ICMS.
    #[serde(rename = "modBC", default)]
    pub mod_bc: Option<String>,
    /// Base de cálculo do ICMS.
    #[serde(rename = "vBC", default)]
    pub v_bc: Option<String>,
    /// Percentual de redução de base de cálculo do ICMS.
    #[serde(rename = "pRedBC", default)]
    pub p_red_bc: Option<String>,
    /// Alíquota do ICMS.
    #[serde(rename = "pICMS", default)]
    pub p_icms: Option<String>,
    /// Valor monetário do ICMS.
    #[serde(rename = "vICMS", default)]
    pub v_icms: Option<String>,
    /// Modalidade de determinação da base de cálculo do ICMS ST.
    #[serde(rename = "modBCST", default)]
    pub mod_bcst: Option<String>,
    /// Margem de valor adicionado do ICMS ST.
    #[serde(rename = "pMVAST", default)]
    pub p_mvast: Option<String>,
    /// Percentual de redução de base de cálculo do ICMS ST.
    #[serde(rename = "pRedBCST", default)]
    pub p_red_bcst: Option<String>,
    /// Base de cálculo do ICMS ST.
    #[serde(rename = "vBCST", default)]
    pub v_bcst: Option<String>,
    /// Alíquota do ICMS ST.
    #[serde(rename = "pICMSST", default)]
    pub p_icmsst: Option<String>,
    /// Valor do ICMS ST.
    #[serde(rename = "vICMSST", default)]
    pub v_icmsst: Option<String>,
    /// Base de cálculo do FCP retido por ST.
    #[serde(rename = "vBCFCPST", default)]
    pub v_bcfcpst: Option<String>,
    /// Alíquota do FCP retido por ST.
    #[serde(rename = "pFCPST", default)]
    pub p_fcpst: Option<String>,
    /// Valor do FCP retido por ST.
    #[serde(rename = "vFCPST", default)]
    pub v_fcpst: Option<String>,
    /// Alíquota aplicável para cálculo do crédito no Simples Nacional.
    #[serde(rename = "pCredSN", default)]
    pub p_cred_sn: Option<String>,
    /// Valor do crédito de ICMS do Simples Nacional.
    #[serde(rename = "vCredICMSSN", default)]
    pub v_cred_icmssn: Option<String>,
    /// Texto descritivo bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// ICMS devido à UF de destino para partilha e Fundo de Combate à Pobreza (`<ICMSUFFim>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Icmsuffim {
    /// Base de cálculo do ICMS na UF de destino.
    #[serde(rename = "vBCUFDest", default)]
    pub v_bcuffim: Option<String>,

    /// Percentual correspondente ao Fundo de Combate à Pobreza (FCP) na UF de destino.
    #[serde(rename = "pFCPUFFim", default)]
    pub p_fcpuffim: Option<String>,

    /// Alíquota interna adotada na UF de destino.
    #[serde(rename = "pICMSUFFim", default)]
    pub p_icmsuffim: Option<String>,

    /// Alíquota interestadual declarada das UFs envolvidas.
    #[serde(rename = "pICMSInter", default)]
    pub p_icmsinter: Option<String>,

    /// Valor calculado correspondente ao FCP para a UF de destino.
    #[serde(rename = "vFCPUFFim", default)]
    pub v_fcpuffim: Option<String>,

    /// Valor calculado do ICMS de partilha interestadual devido à UF de destino.
    #[serde(rename = "vICMSUFFim", default)]
    pub v_icmsuffim: Option<String>,

    /// Valor calculado do ICMS de partilha interestadual devido à UF de origem.
    #[serde(rename = "vICMSUFIni", default)]
    pub v_icmsufini: Option<String>,

    /// Texto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    /// Percentual provisório de partilha interestadual.
    #[serde(rename = "pICMSInterPart", default)]
    pub p_icmsinter_part: Option<String>,
}

/// Imposto de Importação (`<II>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Ii {
    /// Base de cálculo do imposto de importação.
    #[serde(rename = "vBC", default)]
    pub v_bc: Option<String>,
    /// Despesas aduaneiras.
    #[serde(rename = "vDespAdu", default)]
    pub v_desp_adu: Option<String>,
    /// Valor do imposto de importação.
    #[serde(rename = "vII", default)]
    pub v_ii: Option<String>,
    /// Valor do IOF.
    #[serde(rename = "vIOF", default)]
    pub v_iof: Option<String>,
    /// Texto descritivo bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// Imposto sobre Produtos Industrializados (`<IPI>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Ipi {
    /// CNPJ do produtor se aplicável.
    #[serde(rename = "CNPJProd", default)]
    pub cnpjprod: Option<String>,
    /// Código do selo de controle.
    #[serde(rename = "cSelo", default)]
    pub c_selo: Option<String>,
    /// Quantidade de selos.
    #[serde(rename = "qSelo", default)]
    pub q_selo: Option<String>,
    /// Código de enquadramento do IPI.
    #[serde(rename = "cEnq", default)]
    pub c_enq: Option<String>,
    /// Texto descritivo bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
    /// Detalhes de enquadramento do IPI tributado.
    #[serde(rename = "IPITrib", default)]
    pub ipitrib: Option<Ipitrib>,
    /// Detalhe do IPI não tributado.
    #[serde(rename = "IPINT", default)]
    pub ipint: Option<Ipint>,
}

impl Ipi {
    /// Obtém o valor do IPI correspondente.
    fn get_valor_ipi(&self) -> Option<f64> {
        self.ipitrib
            .as_ref()
            .and_then(|trib| trib.v_ipi.parse_opt())
    }
}

/// IPI Tributável (`<IPITrib>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Ipitrib {
    /// Código de Situação Tributária.
    #[serde(rename = "CST", default)]
    pub cst: Option<String>,
    /// Base de cálculo do IPI.
    #[serde(rename = "vBC", default)]
    pub v_bc: Option<String>,
    /// Alíquota do IPI.
    #[serde(rename = "pIPI", default)]
    pub p_ipi: Option<String>,
    /// Quantidade de produto vendida se pautado por valor.
    #[serde(rename = "qUnid", default)]
    pub q_unid: Option<String>,
    /// Valor por unidade.
    #[serde(rename = "vUnid", default)]
    pub v_unid: Option<String>,
    /// Valor monetário calculado do IPI.
    #[serde(rename = "vIPI", default)]
    pub v_ipi: Option<String>,
    /// Texto descritivo bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// IPI Não Tributável (`<IPINT>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Ipint {
    /// Código de Situação Tributária (CST).
    #[serde(rename = "CST", default)]
    pub cst: Option<String>,
    /// Texto descritivo bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// Imposto Sobre Serviços de Qualquer Natureza (`<ISSQN>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Issqn {
    /// Base de cálculo do ISSQN.
    #[serde(rename = "vBC", default)]
    pub v_bc: Option<String>,
    /// Alíquota do ISSQN.
    #[serde(rename = "vAliq", default)]
    pub v_aliq: Option<String>,
    /// Valor do ISSQN.
    #[serde(rename = "vISSQN", default)]
    pub v_issqn: Option<String>,
    /// Código do Município de ocorrência do fato gerador do ISSQN.
    #[serde(rename = "cMunFG", default)]
    pub c_mun_fg: Option<String>,
    /// Código da lista de serviços.
    #[serde(rename = "cListServ", default)]
    pub c_list_serv: Option<String>,
    /// Valor de dedução se aplicável.
    #[serde(rename = "vDeducao", default)]
    pub v_deducao: Option<String>,
    /// Valor de outras desonerações.
    #[serde(rename = "vOutro", default)]
    pub v_outro: Option<String>,
    /// Valor do desconto incondicionado.
    #[serde(rename = "vDescIncond", default)]
    pub v_desc_incond: Option<String>,
    /// Valor do desconto condicionado.
    #[serde(rename = "vDescCond", default)]
    pub v_desc_cond: Option<String>,
    /// Valor do ISS retido.
    #[serde(rename = "vISSRet", default)]
    pub v_issret: Option<String>,
    /// Indicador de exigibilidade do ISS.
    #[serde(rename = "indISS", default)]
    pub ind_iss: Option<String>,
    /// Código do serviço.
    #[serde(rename = "cServico", default)]
    pub c_servico: Option<String>,
    /// Código do Município.
    #[serde(rename = "cMun", default)]
    pub c_mun: Option<String>,
    /// Código do País.
    #[serde(rename = "cPais", default)]
    pub c_pais: Option<String>,
    /// Número do processo se aplicável.
    #[serde(rename = "nProcesso", default)]
    pub n_processo: Option<String>,
    /// Indicador de incentivo fiscal.
    #[serde(rename = "indIncentivo", default)]
    pub ind_incentivo: Option<String>,
    /// Texto descritivo bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

impl Issqn {
    /// Obtém o valor monetário apurado do ISS.
    fn get_valor_iss(&self) -> Option<f64> {
        self.v_issqn.parse_opt()
    }
}

/// Bloco correspondente ao imposto PIS (`<PIS>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Pis {
    /// Texto descritivo bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
    /// Detalhamento de PIS tributado por alíquota percentual.
    #[serde(rename = "PISAliq", default)]
    pub pisaliq: Option<Pisaliq>,
    /// Detalhamento de PIS tributado por alíquota em valor (pauta).
    #[serde(rename = "PISQtde", default)]
    pub pisqtde: Option<Pisqtde>,
    /// Detalhamento de PIS não tributado.
    #[serde(rename = "PISNT", default)]
    pub pisnt: Option<Pisnt>,
    /// Detalhamento de PIS outras operações.
    #[serde(rename = "PISOutr", default)]
    pub pisoutr: Option<Pisoutr>,
}

impl Pis {
    /// Obtém de forma declarativa o CST do PIS.
    pub fn get_pis_cst(&self) -> Option<u8> {
        self.pisaliq
            .as_ref()
            .and_then(|p| p.cst.parse_opt())
            .or_else(|| self.pisqtde.as_ref().and_then(|p| p.cst.parse_opt()))
            .or_else(|| self.pisnt.as_ref().and_then(|p| p.cst.parse_opt()))
            .or_else(|| self.pisoutr.as_ref().and_then(|p| p.cst.parse_opt()))
    }

    /// Obtém de forma declarativa a alíquota de PIS aplicável.
    pub fn get_pis_aliquota(&self) -> Option<f64> {
        self.pisaliq
            .as_ref()
            .and_then(|p| p.p_pis.parse_opt())
            .or_else(|| {
                self.pisqtde
                    .as_ref()
                    .and_then(|p| p.v_aliq_prod.parse_opt())
            })
            .or_else(|| self.pisoutr.as_ref().and_then(|p| p.p_pis.parse_opt()))
    }

    /// Obtém de forma declarativa o valor monetário calculado do PIS.
    pub fn get_pis_valor(&self) -> Option<f64> {
        self.pisaliq
            .as_ref()
            .and_then(|p| p.v_pis.parse_opt())
            .or_else(|| self.pisqtde.as_ref().and_then(|p| p.v_pis.parse_opt()))
            .or_else(|| self.pisoutr.as_ref().and_then(|p| p.v_pis.parse_opt()))
    }
}

/// PIS tributado por alíquota percentual (`<PISAliq>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Pisaliq {
    /// Código de Situação Tributária (CST).
    #[serde(rename = "CST", default)]
    pub cst: Option<String>,
    /// Base de cálculo.
    #[serde(rename = "vBC", default)]
    pub v_bc: Option<String>,
    /// Alíquota do PIS em percentual.
    #[serde(rename = "pPIS", default)]
    pub p_pis: Option<String>,
    /// Valor monetário calculado de PIS.
    #[serde(rename = "vPIS", default)]
    pub v_pis: Option<String>,
    /// Texto descritivo bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// PIS tributado por alíquota por unidade de produto (pauta) (`<PISQtde>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Pisqtde {
    /// Código de Situação Tributária (CST).
    #[serde(rename = "CST", default)]
    pub cst: Option<String>,
    /// Quantidade de base de cálculo em produto.
    #[serde(rename = "qBCProd", default)]
    pub q_bcprod: Option<String>,
    /// Valor da alíquota por produto.
    #[serde(rename = "vAliqProd", default)]
    pub v_aliq_prod: Option<String>,
    /// Valor de PIS calculado.
    #[serde(rename = "vPIS", default)]
    pub v_pis: Option<String>,
    /// Texto descritivo bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// PIS não tributável (`<PISNT>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Pisnt {
    /// Código de Situação Tributária (CST).
    #[serde(rename = "CST", default)]
    pub cst: Option<String>,
    /// Texto descritivo bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// PIS outras operações (`<PISOutr>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Pisoutr {
    /// Código de Situação Tributária (CST).
    #[serde(rename = "CST", default)]
    pub cst: Option<String>,
    /// Base de cálculo.
    #[serde(rename = "vBC", default)]
    pub v_bc: Option<String>,
    /// Alíquota do PIS em percentual.
    #[serde(rename = "pPIS", default)]
    pub p_pis: Option<String>,
    /// Quantidade base de cálculo.
    #[serde(rename = "qBCProd", default)]
    pub q_bcprod: Option<String>,
    /// Valor da alíquota por produto.
    #[serde(rename = "vAliqProd", default)]
    pub v_aliq_prod: Option<String>,
    /// Valor do PIS calculado.
    #[serde(rename = "vPIS", default)]
    pub v_pis: Option<String>,
    /// Texto descritivo bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// PIS Substituição Tributária (`<PISST>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Pisst {
    /// Base de cálculo do PIS ST.
    #[serde(rename = "vBC", default)]
    pub v_bc: Option<String>,
    /// Alíquota percentual do PIS ST.
    #[serde(rename = "pPIS", default)]
    pub p_pis: Option<String>,
    /// Quantidade de produto base de cálculo.
    #[serde(rename = "qBCProd", default)]
    pub q_bcprod: Option<String>,
    /// Valor de alíquota por produto.
    #[serde(rename = "vAliqProd", default)]
    pub v_aliq_prod: Option<String>,
    /// Valor calculado de PIS ST.
    #[serde(rename = "vPIS", default)]
    pub v_pis: Option<String>,
    /// Texto descritivo bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// Bloco correspondente ao imposto COFINS (`<COFINS>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Cofins {
    /// Texto descritivo bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
    /// Detalhamento de COFINS tributado por alíquota percentual.
    #[serde(rename = "COFINSAliq", default)]
    pub cofinsaliq: Option<Cofinsaliq>,
    /// Detalhamento de COFINS tributado por alíquota em valor (pauta).
    #[serde(rename = "COFINSQtde", default)]
    pub cofinsqtde: Option<Cofinsqtde>,
    /// Detalhamento de COFINS não tributado.
    #[serde(rename = "COFINSNT", default)]
    pub cofinsnt: Option<Cofinsnt>,
    /// Detalhamento de COFINS outras operações.
    #[serde(rename = "COFINSOutr", default)]
    pub cofinsoutr: Option<Cofinsoutr>,
}

impl Cofins {
    /// Obtém de forma declarativa o CST de COFINS.
    pub fn get_cofins_cst(&self) -> Option<u8> {
        self.cofinsaliq
            .as_ref()
            .and_then(|c| c.cst.parse_opt())
            .or_else(|| self.cofinsqtde.as_ref().and_then(|c| c.cst.parse_opt()))
            .or_else(|| self.cofinsnt.as_ref().and_then(|c| c.cst.parse_opt()))
            .or_else(|| self.cofinsoutr.as_ref().and_then(|c| c.cst.parse_opt()))
    }

    /// Obtém de forma declarativa a alíquota aplicável de COFINS.
    pub fn get_cofins_aliquota(&self) -> Option<f64> {
        self.cofinsaliq
            .as_ref()
            .and_then(|c| c.p_cofins.parse_opt())
            .or_else(|| {
                self.cofinsqtde
                    .as_ref()
                    .and_then(|c| c.v_aliq_prod.parse_opt())
            })
            .or_else(|| {
                self.cofinsoutr
                    .as_ref()
                    .and_then(|c| c.p_cofins.parse_opt())
            })
    }

    /// Obtém de forma declarativa o valor monetário calculado do COFINS.
    pub fn get_cofins_valor(&self) -> Option<f64> {
        self.cofinsaliq
            .as_ref()
            .and_then(|c| c.v_cofins.parse_opt())
            .or_else(|| {
                self.cofinsqtde
                    .as_ref()
                    .and_then(|c| c.v_cofins.parse_opt())
            })
            .or_else(|| {
                self.cofinsoutr
                    .as_ref()
                    .and_then(|c| c.v_cofins.parse_opt())
            })
    }
}

/// COFINS tributado por alíquota percentual (`<COFINSAliq>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Cofinsaliq {
    /// Código de Situação Tributária (CST).
    #[serde(rename = "CST", default)]
    pub cst: Option<String>,
    /// Base de cálculo.
    #[serde(rename = "vBC", default)]
    pub v_bc: Option<String>,
    /// Alíquota de COFINS em percentual.
    #[serde(rename = "pCOFINS", default)]
    pub p_cofins: Option<String>,
    /// Valor calculado de COFINS.
    #[serde(rename = "vCOFINS", default)]
    pub v_cofins: Option<String>,
    /// Texto descritivo bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// COFINS tributado por pauta unitária (`<COFINSQtde>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Cofinsqtde {
    /// Código de Situação Tributária (CST).
    #[serde(rename = "CST", default)]
    pub cst: Option<String>,
    /// Quantidade base de cálculo.
    #[serde(rename = "qBCProd", default)]
    pub q_bcprod: Option<String>,
    /// Alíquota por unidade do produto.
    #[serde(rename = "vAliqProd", default)]
    pub v_aliq_prod: Option<String>,
    /// Valor calculado de COFINS.
    #[serde(rename = "vCOFINS", default)]
    pub v_cofins: Option<String>,
    /// Texto descritivo bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// COFINS não tributável (`<COFINSNT>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Cofinsnt {
    /// Código de Situação Tributária (CST).
    #[serde(rename = "CST", default)]
    pub cst: Option<String>,
    /// Texto descritivo bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// COFINS outras operações (`<COFINSOutr>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Cofinsoutr {
    /// Código de Situação Tributária (CST).
    #[serde(rename = "CST", default)]
    pub cst: Option<String>,
    /// Base de cálculo.
    #[serde(rename = "vBC", default)]
    pub v_bc: Option<String>,
    /// Alíquota percentual do COFINS.
    #[serde(rename = "pCOFINS", default)]
    pub p_cofins: Option<String>,
    /// Quantidade base de cálculo.
    #[serde(rename = "qBCProd", default)]
    pub q_bcprod: Option<String>,
    /// Alíquota por produto.
    #[serde(rename = "vAliqProd", default)]
    pub v_aliq_prod: Option<String>,
    /// Valor calculado de COFINS.
    #[serde(rename = "vCOFINS", default)]
    pub v_cofins: Option<String>,
    /// Texto descritivo bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// COFINS Substituição Tributária (`<COFINSST>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Cofinsst {
    /// Base de cálculo do COFINS ST.
    #[serde(rename = "vBC", default)]
    pub v_bc: Option<String>,
    /// Alíquota percentual do COFINS ST.
    #[serde(rename = "pCOFINS", default)]
    pub p_cofins: Option<String>,
    /// Quantidade de produto base de cálculo.
    #[serde(rename = "qBCProd", default)]
    pub q_bcprod: Option<String>,
    /// Valor da alíquota por produto.
    #[serde(rename = "vAliqProd", default)]
    pub v_aliq_prod: Option<String>,
    /// Valor calculado do COFINS ST.
    #[serde(rename = "vCOFINS", default)]
    pub v_cofins: Option<String>,
    /// Texto descritivo bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// Bloco do ICMS para UF de Destino (`<ICMSUFDest>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Icmsufdest {
    /// Base de cálculo do ICMS na UF de destino.
    #[serde(rename = "vBCUFDest", default)]
    pub v_bcufdest: Option<String>,
    /// Base de cálculo do FCP na UF de destino.
    #[serde(rename = "vBCFCPUFDest", default)]
    pub v_bcfcpufdest: Option<String>,
    /// Alíquota do FCP na UF de destino.
    #[serde(rename = "pFCPUFDest", default)]
    pub p_fcpufdest: Option<String>,
    /// Alíquota interna do ICMS na UF de destino.
    #[serde(rename = "pICMSUFDest", default)]
    pub p_icmsufdest: Option<String>,
    /// Alíquota interestadual das UFs envolvidas.
    #[serde(rename = "pICMSInter", default)]
    pub p_icmsinter: Option<String>,
    /// Percentual de partilha do ICMS interestadual.
    #[serde(rename = "pICMSInterPart", default)]
    pub p_icmsinter_part: Option<String>,
    /// Valor monetário calculado do FCP na UF de destino.
    #[serde(rename = "vFCPUFDest", default)]
    pub v_fcpufdest: Option<String>,
    /// Valor do ICMS na UF de destino.
    #[serde(rename = "vICMSUFDest", default)]
    pub v_icmsufdest: Option<String>,
    /// Valor do ICMS na UF de origem (remetente).
    #[serde(rename = "vICMSUFRemet", default)]
    pub v_icmsufremet: Option<String>,
    /// Texto descritivo bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// Bloco consolidador de tributos e impostos incidentes sobre o item (`<total>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Total {
    /// Texto descritivo bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    /// Totais consolidados relativos ao ICMS.
    #[serde(rename = "ICMSTot", default)]
    pub icmstot: Option<IcmsTot>,

    /// Totais consolidados relativos ao ISSQN.
    #[serde(rename = "ISSQNtot", default)]
    pub issqntot: Option<Issqntot>,

    /// Retenções tributárias consolidadas.
    #[serde(rename = "retTrib", default)]
    pub ret_trib: Option<RetTrib>,

    /// Total consolidado do IBS e da CBS (IBSCBSTot).
    #[serde(rename = "IBSCBSTot", default)]
    pub ibscbstot: Option<Ibscbstot>,

    /// Valor Total da NF-e incluindo possíveis impostos novos e ajustes (vNFTot).
    #[serde(rename = "vNFTot", default)]
    pub v_nf_tot: Option<String>,
}

impl Total {
    /// Obtém o valor monetário bruto total da Nota Fiscal (vNF).
    pub fn get_valor_nfe(&self) -> Option<f64> {
        self.icmstot
            .as_ref()
            .and_then(|icms_total| icms_total.v_nf.parse_opt())
    }
}

/// Totais de ICMS consolidados da NF-e (`<ICMSTot>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IcmsTot {
    /// Base de cálculo global do ICMS.
    #[serde(rename = "vBC", default)]
    pub v_bc: Option<String>,
    /// Valor total de ICMS apurado.
    #[serde(rename = "vICMS", default)]
    pub v_icms: Option<String>,
    /// Valor total de ICMS desonerado apurado.
    #[serde(rename = "vICMSDeson", default)]
    pub v_icmsdeson: Option<String>,
    /// Valor total do FCP devido na UF de destino.
    #[serde(rename = "vFCPUFDest", default)]
    pub v_fcpufdest: Option<String>,
    /// Valor total do ICMS diferido (vICMSDif).
    #[serde(rename = "vICMSDif", default)]
    pub v_icms_dif: Option<String>,
    /// Valor total do ICMS devido na UF de destino.
    #[serde(rename = "vICMSUFDest", default)]
    pub v_icmsufdest: Option<String>,
    /// Valor total do ICMS devido na UF de origem (remetente).
    #[serde(rename = "vICMSUFRemet", default)]
    pub v_icmsufremet: Option<String>,
    /// Valor total do FCP.
    #[serde(rename = "vFCP", default)]
    pub v_fcp: Option<String>,
    /// Base de cálculo global do ICMS ST.
    #[serde(rename = "vBCST", default)]
    pub v_bcst: Option<String>,
    /// Valor total de ICMS retido por Substituição Tributária.
    #[serde(rename = "vST", default)]
    pub v_st: Option<String>,
    /// Valor total do FCP retido por ST.
    #[serde(rename = "vFCPST", default)]
    pub v_fcpst: Option<String>,
    /// Valor total do FCP retido anteriormente por ST.
    #[serde(rename = "vFCPSTRet", default)]
    pub v_fcpstret: Option<String>,
    /// Valor total consolidado dos produtos faturados.
    #[serde(rename = "vProd", default)]
    pub v_prod: Option<String>,
    /// Valor total do frete.
    #[serde(rename = "vFrete", default)]
    pub v_frete: Option<String>,
    /// Valor total do seguro.
    #[serde(rename = "vSeg", default)]
    pub v_seg: Option<String>,
    /// Valor total de descontos.
    #[serde(rename = "vDesc", default)]
    pub v_desc: Option<String>,
    /// Valor total do imposto de importação.
    #[serde(rename = "vII", default)]
    pub v_ii: Option<String>,
    /// Valor total de IPI.
    #[serde(rename = "vIPI", default)]
    pub v_ipi: Option<String>,
    /// Valor total de IPI devolvido.
    #[serde(rename = "vIPIDevol", default)]
    pub v_ipidevol: Option<String>,
    /// Valor total do PIS.
    #[serde(rename = "vPIS", default)]
    pub v_pis: Option<String>,
    /// Valor total do COFINS.
    #[serde(rename = "vCOFINS", default)]
    pub v_cofins: Option<String>,
    /// Valor total de despesas acessórias.
    #[serde(rename = "vOutro", default)]
    pub v_outro: Option<String>,
    /// Valor monetário líquido total da Nota Fiscal (vNF).
    #[serde(rename = "vNF", default)]
    pub v_nf: Option<String>,
    /// Valor estimado de tributos incidentes consolidado.
    #[serde(rename = "vTotTrib", default)]
    pub v_tot_trib: Option<String>,
    /// Quantidade global de base de cálculo monofásica do ICMS (combustíveis).
    #[serde(rename = "qBCMono", default)]
    pub q_bc_mono: Option<String>,
    /// Valor global do ICMS monofásico próprio.
    #[serde(rename = "vICMSMono", default)]
    pub v_icms_mono: Option<String>,
    /// Quantidade global de base de cálculo monofásica retida.
    #[serde(rename = "qBCMonoReten", default)]
    pub q_bc_mono_reten: Option<String>,
    /// Valor global de ICMS monofásico retido.
    #[serde(rename = "vICMSMonoReten", default)]
    pub v_icms_mono_reten: Option<String>,
    /// Quantidade global de base de cálculo monofásica retida anteriormente.
    #[serde(rename = "qBCMonoRet", default)]
    pub q_bc_mono_ret: Option<String>,
    /// Valor global de ICMS monofásico retido anteriormente.
    #[serde(rename = "vICMSMonoRet", default)]
    pub v_icms_mono_ret: Option<String>,
    /// Texto descritivo bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// Totais de ISSQN consolidados da NF-e (`<ISSQNtot>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Issqntot {
    /// Valor monetário total correspondente aos serviços prestados.
    #[serde(rename = "vServ", default)]
    pub v_serv: Option<String>,
    /// Base de cálculo global do ISSQN.
    #[serde(rename = "vBC", default)]
    pub v_bc: Option<String>,
    /// Valor total de ISS apurado.
    #[serde(rename = "vISS", default)]
    pub v_iss: Option<String>,
    /// Valor total de PIS apurado sobre serviços.
    #[serde(rename = "vPIS", default)]
    pub v_pis: Option<String>,
    /// Valor total de COFINS apurado sobre serviços.
    #[serde(rename = "vCOFINS", default)]
    pub v_cofins: Option<String>,
    /// Data de competência.
    #[serde(rename = "dCompet", default)]
    pub d_compet: Option<String>,
    /// Valor total de dedução.
    #[serde(rename = "vDeducao", default)]
    pub v_deducao: Option<String>,
    /// Valor total de outras desonerações.
    #[serde(rename = "vOutro", default)]
    pub v_outro: Option<String>,
    /// Valor do desconto incondicionado.
    #[serde(rename = "vDescIncond", default)]
    pub v_desc_incond: Option<String>,
    /// Valor do desconto condicionado.
    #[serde(rename = "vDescCond", default)]
    pub v_desc_cond: Option<String>,
    /// Valor total de ISS retido.
    #[serde(rename = "vISSRet", default)]
    pub v_issret: Option<String>,
    /// Código do regime tributário de ISSQN municipal.
    #[serde(rename = "cRegTrib", default)]
    pub c_reg_trib: Option<String>,
    /// Texto descritivo bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// Grupo de Retenções Tributárias consolidadas da NF-e (`<retTrib>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RetTrib {
    /// Valor total do PIS retido na fonte.
    #[serde(rename = "vRetPIS", default)]
    pub v_ret_pis: Option<String>,
    /// Valor total do COFINS retido na fonte.
    #[serde(rename = "vRetCOFINS", default)]
    pub v_ret_cofins: Option<String>,
    /// Valor total do CSLL retido na fonte.
    #[serde(rename = "vRetCSLL", default)]
    pub v_ret_csll: Option<String>,
    /// Base de cálculo global de IRRF.
    #[serde(rename = "vBCIRRF", default)]
    pub v_bcirrf: Option<String>,
    /// Valor total de IRRF retido.
    #[serde(rename = "vIRRF", default)]
    pub v_irrf: Option<String>,
    /// Base de cálculo global de contribuição previdenciária.
    #[serde(rename = "vBCRetPrev", default)]
    pub v_bcret_prev: Option<String>,
    /// Valor total de retenção previdenciária.
    #[serde(rename = "vRetPrev", default)]
    pub v_ret_prev: Option<String>,
    /// Texto descritivo bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// Total consolidado do IBS e da CBS (`<IBSCBSTot>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Ibscbstot {
    /// Base de cálculo consolidada do IBS e da CBS (vBCIBSCBS).
    #[serde(rename = "vBCIBSCBS", default)]
    pub v_bc_ibscbs: Option<String>,

    /// Grupo de totais consolidados do IBS (gIBS).
    #[serde(rename = "gIBS", default)]
    pub g_ibs: Option<GIbsTot>,

    /// Grupo de totais consolidados da CBS (gCBS).
    #[serde(rename = "gCBS", default)]
    pub g_cbs: Option<GCbsTot>,

    /// Conteúdo de texto se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// Grupo de totais consolidados do IBS (`<gIBS>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GIbsTot {
    /// Valor consolidado do IBS.
    #[serde(rename = "vIBS", default)]
    pub v_ibs: Option<String>,

    /// Valor do crédito presumido do IBS.
    #[serde(rename = "vCredPres", default)]
    pub v_cred_pres: Option<String>,

    /// Valor do crédito presumido sob condição suspensiva do IBS.
    #[serde(rename = "vCredPresCondSus", default)]
    pub v_cred_pres_cond_sus: Option<String>,

    /// Detalhamento dos totais do IBS Estadual (gIBSUF).
    #[serde(rename = "gIBSUF", default)]
    pub g_ibs_uf: Option<GIbsUfTot>,

    /// Detalhamento dos totais do IBS Municipal (gIBSMun).
    #[serde(rename = "gIBSMun", default)]
    pub g_ibs_mun: Option<GIbsMunTot>,

    /// Conteúdo de texto se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// Detalhamento dos totais do IBS Estadual (`<gIBSUF>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GIbsUfTot {
    /// Valor consolidado do IBS Estadual (UF).
    #[serde(rename = "vIBSUF", default)]
    pub v_ibs_uf: Option<String>,

    /// Valor do IBS Estadual diferido (vDif).
    #[serde(rename = "vDif", default)]
    pub v_dif: Option<String>,

    /// Valor do IBS Estadual em devolução de tributo (vDevTrib).
    #[serde(rename = "vDevTrib", default)]
    pub v_dev_trib: Option<String>,

    /// Conteúdo de texto se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// Detalhamento dos totais do IBS Municipal (`<gIBSMun>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GIbsMunTot {
    /// Valor consolidado do IBS Municipal.
    #[serde(rename = "vIBSMun", default)]
    pub v_ibs_mun: Option<String>,

    /// Valor do IBS Municipal diferido (vDif).
    #[serde(rename = "vDif", default)]
    pub v_dif: Option<String>,

    /// Valor do IBS Municipal em devolução de tributo (vDevTrib).
    #[serde(rename = "vDevTrib", default)]
    pub v_dev_trib: Option<String>,

    /// Conteúdo de texto se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// Grupo de totais consolidados da CBS (`<gCBS>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GCbsTot {
    /// Valor consolidado da CBS (vCBS).
    #[serde(rename = "vCBS", default)]
    pub v_cbs: Option<String>,

    /// Valor do crédito presumido da CBS.
    #[serde(rename = "vCredPres", default)]
    pub v_cred_pres: Option<String>,

    /// Valor do crédito presumido sob condição suspensiva da CBS.
    #[serde(rename = "vCredPresCondSus", default)]
    pub v_cred_pres_cond_sus: Option<String>,

    /// Valor da CBS diferida (vDif).
    #[serde(rename = "vDif", default)]
    pub v_dif: Option<String>,

    /// Valor da CBS em devolução de tributo (vDevTrib).
    #[serde(rename = "vDevTrib", default)]
    pub v_dev_trib: Option<String>,

    /// Conteúdo de texto se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// Detalhes do Imposto sobre Bens e Serviços (`<IBS>`) - IVA Estadual/Municipal.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Ibs {
    /// Código de Situação Tributária (CST) correspondente ao IBS.
    #[serde(rename = "CST", default)]
    pub cst: Option<String>,

    /// Base de cálculo apurada do IBS.
    #[serde(rename = "vBC", default)]
    pub v_bc: Option<String>,

    /// Alíquota percentual do IBS.
    #[serde(rename = "pIBS", default)]
    pub p_ibs: Option<String>,

    /// Valor calculado do IBS.
    #[serde(rename = "vIBS", default)]
    pub v_ibs: Option<String>,

    /// Código do Município de ocorrência do fato gerador do IBS.
    #[serde(rename = "cMunFGIBS", default)]
    pub c_mun_fg_ibs: Option<String>,

    /// Texto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

impl Ibs {
    /// Recupera o CST do IBS convertido para número.
    pub fn get_ibs_cst(&self) -> Option<u8> {
        self.cst.parse_opt()
    }

    /// Recupera a alíquota percentual do IBS.
    pub fn get_ibs_aliquota(&self) -> Option<f64> {
        self.p_ibs.parse_opt()
    }

    /// Recupera o valor monetário calculado do IBS.
    pub fn get_ibs_valor(&self) -> Option<f64> {
        self.v_ibs.parse_opt()
    }
}

/// Detalhes da Contribuição sobre Bens e Serviços (`<CBS>`) - IVA Federal.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Cbs {
    /// Código de Situação Tributária (CST) correspondente à CBS.
    #[serde(rename = "CST", default)]
    pub cst: Option<String>,

    /// Base de cálculo apurada da CBS.
    #[serde(rename = "vBC", default)]
    pub v_bc: Option<String>,

    /// Alíquota percentual da CBS.
    #[serde(rename = "pCBS", default)]
    pub p_cbs: Option<String>,

    /// Valor calculado da CBS.
    #[serde(rename = "vCBS", default)]
    pub v_cbs: Option<String>,

    /// Texto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

impl Cbs {
    /// Recupera o CST da CBS convertido para número.
    pub fn get_cbs_cst(&self) -> Option<u8> {
        self.cst.parse_opt()
    }

    /// Recupera a alíquota percentual da CBS.
    pub fn get_cbs_aliquota(&self) -> Option<f64> {
        self.p_cbs.parse_opt()
    }

    /// Recupera o valor monetário calculado da CBS.
    pub fn get_cbs_valor(&self) -> Option<f64> {
        self.v_cbs.parse_opt()
    }
}

/// Grupo de Tributação do IBS e da CBS nos itens (`<IBSCBS>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Ibscbs {
    /// Código de classificação tributária do IBS e da CBS.
    #[serde(rename = "cClassTrib", default)]
    pub c_class_trib: Option<String>,

    /// Conteúdo de texto se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// Detalhes do Imposto Seletivo (`<IS>`) - Incidência extrafiscal sobre produtos específicos.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ImpostoSeletivo {
    /// Código de Situação Tributária (CST) correspondente ao Imposto Seletivo.
    #[serde(rename = "CST", default)]
    pub cst: Option<String>,

    /// Base de cálculo apurada do Imposto Seletivo.
    #[serde(rename = "vBC", default)]
    pub v_bc: Option<String>,

    /// Alíquota percentual do Imposto Seletivo.
    #[serde(rename = "pIS", default)]
    pub p_is: Option<String>,

    /// Valor calculado do Imposto Seletivo.
    #[serde(rename = "vIS", default)]
    pub v_is: Option<String>,

    /// Quantidade de produto correspondente para base de cálculo em pauta se aplicável.
    #[serde(rename = "qBCProd", default)]
    pub q_bc_prod: Option<String>,

    /// Valor da alíquota em pauta do produto se aplicável.
    #[serde(rename = "vAliqProd", default)]
    pub v_aliq_prod: Option<String>,

    /// Texto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

impl ImpostoSeletivo {
    /// Recupera o CST do Imposto Seletivo convertido para número.
    pub fn get_is_cst(&self) -> Option<u8> {
        self.cst.parse_opt()
    }

    /// Recupera o valor monetário calculado do Imposto Seletivo.
    pub fn get_is_valor(&self) -> Option<f64> {
        self.v_is.parse_opt()
    }
}

/// Tributação monofásica própria sobre combustíveis (CST 02).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Icms02 {
    /// Origem da mercadoria.
    #[serde(rename = "orig", default)]
    pub orig: Option<String>,
    /// Código de Situação Tributária (CST 02).
    #[serde(rename = "CST", default)]
    pub cst: Option<String>,
    /// Quantidade de base de cálculo monofásica própria.
    #[serde(rename = "qBCMono", default)]
    pub q_bc_mono: Option<String>,
    /// Valor da alíquota ad rem própria do ICMS.
    #[serde(rename = "adRemICMS", default)]
    pub ad_rem_icms: Option<String>,
    /// Valor monetário calculado do ICMS monofásico próprio.
    #[serde(rename = "vICMSMono", default)]
    pub v_icms_mono: Option<String>,
    /// Texto descritivo bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// Tributação monofásica própria e com responsabilidade por substituição tributária (CST 15).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Icms15 {
    /// Origem da mercadoria.
    #[serde(rename = "orig", default)]
    pub orig: Option<String>,
    /// Código de Situação Tributária (CST 15).
    #[serde(rename = "CST", default)]
    pub cst: Option<String>,
    /// Quantidade de base de cálculo monofásica própria.
    #[serde(rename = "qBCMono", default)]
    pub q_bc_mono: Option<String>,
    /// Valor da alíquota ad rem própria do ICMS.
    #[serde(rename = "adRemICMS", default)]
    pub ad_rem_icms: Option<String>,
    /// Valor monetário calculado do ICMS monofásico próprio.
    #[serde(rename = "vICMSMono", default)]
    pub v_icms_mono: Option<String>,
    /// Quantidade de base de cálculo monofásica de ST.
    #[serde(rename = "qBCMonoReten", default)]
    pub q_bc_mono_reten: Option<String>,
    /// Valor da alíquota ad rem de ST do ICMS.
    #[serde(rename = "adRemICMSReten", default)]
    pub ad_rem_icms_reten: Option<String>,
    /// Valor do ICMS monofásico de ST.
    #[serde(rename = "vICMSMonoReten", default)]
    pub v_icms_mono_reten: Option<String>,
    /// Texto descritivo bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// Regime monofásico diferido de combustíveis (CST 53).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Icms53 {
    /// Origem da mercadoria.
    #[serde(rename = "orig", default)]
    pub orig: Option<String>,
    /// Código de Situação Tributária (CST 53).
    #[serde(rename = "CST", default)]
    pub cst: Option<String>,
    /// Quantidade de base de cálculo monofásica do diferimento.
    #[serde(rename = "qBCMono", default)]
    pub q_bc_mono: Option<String>,
    /// Alíquota ad rem do diferimento.
    #[serde(rename = "adRemICMS", default)]
    pub ad_rem_icms: Option<String>,
    /// Valor do ICMS monofásico de operação própria.
    #[serde(rename = "vICMSMonoOp", default)]
    pub v_icms_mono_op: Option<String>,
    /// Percentual de diferimento.
    #[serde(rename = "pDif", default)]
    pub p_dif: Option<String>,
    /// Valor do ICMS monofásico diferido.
    #[serde(rename = "vICMSMonoDif", default)]
    pub v_icms_mono_dif: Option<String>,
    /// Valor líquido do ICMS monofásico devido.
    #[serde(rename = "vICMSMono", default)]
    pub v_icms_mono: Option<String>,
    /// Texto descritivo bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// Tributação monofásica sobre combustíveis cobrada anteriormente (CST 61).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Icms61 {
    /// Origem da mercadoria.
    #[serde(rename = "orig", default)]
    pub orig: Option<String>,
    /// Código de Situação Tributária (CST 61).
    #[serde(rename = "CST", default)]
    pub cst: Option<String>,
    /// Quantidade de base de cálculo monofásica retida anteriormente.
    #[serde(rename = "qBCMonoRet", default)]
    pub q_bc_mono_ret: Option<String>,
    /// Valor da alíquota ad rem retida anteriormente.
    #[serde(rename = "adRemICMSRet", default)]
    pub ad_rem_icms_ret: Option<String>,
    /// Valor do ICMS monofásico retido anteriormente.
    #[serde(rename = "vICMSMonoRet", default)]
    pub v_icms_mono_ret: Option<String>,
    /// Texto descritivo bruto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

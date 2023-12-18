#![allow(clippy::upper_case_acronyms)]

use serde::{Deserialize, Serialize};

// Regex:
// :\s+(\w+),
// : Option<$1>,

// Impostos: IPI. ICMS, II, PIS e COFINS

#[derive(Debug, Serialize, Deserialize)]
pub struct Imposto {
    #[serde(rename = "ICMS")]
    pub icms: Option<Icms>,
    #[serde(rename = "ICMSUFFim")]
    pub icmsuffim: Option<Icmsuffim>,
    #[serde(rename = "ICMSUFDest")]
    pub icmsufdest: Option<Icmsufdest>,
    #[serde(rename = "II")]
    pub ii: Option<Ii>,
    #[serde(rename = "IPI")]
    pub ipi: Option<Vec<Ipi>>,
    #[serde(rename = "ISSQN")]
    pub issqn: Option<Issqn>,
    #[serde(rename = "PIS")]
    pub pis: Option<Pis>,
    #[serde(rename = "PISST")]
    pub pisst: Option<Pisst>,
    #[serde(rename = "COFINS")]
    pub cofins: Option<Cofins>,
    #[serde(rename = "COFINSST")]
    pub cofinsst: Option<Cofinsst>,
    #[serde(rename = "infAdFisco")]
    pub inf_ad_fisco: Option<String>,
    #[serde(rename = "vTotTrib")]
    pub v_tot_trib: Option<String>,
}

impl Imposto {
    pub fn get_pis_outr(&self) -> Option<&Pisoutr> {
        self
            .pis
            .as_ref()
            .and_then(|p| {
                p.pisoutr.as_ref()
            })
    }

    pub fn get_v_pis(&self) -> Option<f64> {
        self
            .get_pis_outr()
            .as_ref()
            .and_then(|outro| {
                outro
                    .v_pis
                    .as_ref()
                    .and_then(|v| v.trim().parse().ok())
            })
    }

    pub fn get_cofins_outr(&self) -> Option<&Cofinsoutr> {
        self
            .cofins
            .as_ref()
            .and_then(|c| {
                c.cofinsoutr.as_ref()
            })
    }

    pub fn get_v_cofins(&self) -> Option<f64> {
        self
            .get_cofins_outr()
            .as_ref()
            .and_then(|outro| {
                outro
                    .v_cofins
                    .as_ref()
                    .and_then(|v| v.trim().parse().ok())
            })
    }

    pub fn get_icms00(&self) -> Option<&Icms00> {
        self
            .icms
            .as_ref()
            .and_then(|i| {
                i.icms00.as_ref()
            })
    }

    pub fn get_v_icms(&self) -> Option<f64> {
        self
            .get_icms00()
            .as_ref()
            .and_then(|outro| {
                outro
                    .v_icms
                    .as_ref()
                    .and_then(|v| v.trim().parse().ok())
            })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Icms {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "ICMS00")]
    pub icms00: Option<Icms00>,
    #[serde(rename = "ICMS10")]
    pub icms10: Option<Icms10>,
    #[serde(rename = "ICMS20")]
    pub icms20: Option<Icms20>,
    #[serde(rename = "ICMS30")]
    pub icms30: Option<Icms30>,
    #[serde(rename = "ICMS40")]
    pub icms40: Option<Icms40>,
    #[serde(rename = "ICMS51")]
    pub icms51: Option<Icms51>,
    #[serde(rename = "ICMS60")]
    pub icms60: Option<Icms60>,
    #[serde(rename = "ICMS70")]
    pub icms70: Option<Icms70>,
    #[serde(rename = "ICMS90")]
    pub icms90: Option<Icms90>,
    #[serde(rename = "ICMSPart")]
    pub icmspart: Option<Icmspart>,
    #[serde(rename = "ICMSST")]
    pub icmsst: Option<Icmsst>,
    #[serde(rename = "ICMSSN101")]
    pub icmssn101: Option<Icmssn101>,
    #[serde(rename = "ICMSSN102")]
    pub icmssn102: Option<Icmssn102>,
    #[serde(rename = "ICMSSN201")]
    pub icmssn201: Option<Icmssn201>,
    #[serde(rename = "ICMSSN202")]
    pub icmssn202: Option<Icmssn202>,
    #[serde(rename = "ICMSSN500")]
    pub icmssn500: Option<Icmssn500>,
    #[serde(rename = "ICMSSN900")]
    pub icmssn900: Option<Icmssn900>,
    #[serde(rename = "ICMSOutraUF")]
    pub icmsoutra_uf: Option<IcmsoutraUf>,
    #[serde(rename = "ICMSSN")]
    pub icmssn: Option<Icmssn>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Icms00 {
    #[serde(rename = "orig")]
    pub orig: Option<String>,
    #[serde(rename = "CST")]
    pub cst: Option<String>,
    #[serde(rename = "modBC")]
    pub mod_bc: Option<String>,
    #[serde(rename = "vBC")]
    pub v_bc: Option<String>,
    #[serde(rename = "pICMS")]
    pub p_icms: Option<String>,
    #[serde(rename = "vICMS")]
    pub v_icms: Option<String>,
    #[serde(rename = "pFCP")]
    pub p_fcp: Option<String>,
    #[serde(rename = "vFCP")]
    pub v_fcp: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Icms10 {
    #[serde(rename = "orig")]
    pub orig: Option<String>,
    #[serde(rename = "CST")]
    pub cst: Option<String>,
    #[serde(rename = "modBC")]
    pub mod_bc: Option<String>,
    #[serde(rename = "vBC")]
    pub v_bc: Option<String>,
    #[serde(rename = "pICMS")]
    pub p_icms: Option<String>,
    #[serde(rename = "vICMS")]
    pub v_icms: Option<String>,
    #[serde(rename = "vBCFCP")]
    pub v_bcfcp: Option<String>,
    #[serde(rename = "pFCP")]
    pub p_fcp: Option<String>,
    #[serde(rename = "vFCP")]
    pub v_fcp: Option<String>,
    #[serde(rename = "modBCST")]
    pub mod_bcst: Option<String>,
    #[serde(rename = "pMVAST")]
    pub p_mvast: Option<String>,
    #[serde(rename = "pRedBCST")]
    pub p_red_bcst: Option<String>,
    #[serde(rename = "vBCST")]
    pub v_bcst: Option<String>,
    #[serde(rename = "pICMSST")]
    pub p_icmsst: Option<String>,
    #[serde(rename = "vICMSST")]
    pub v_icmsst: Option<String>,
    #[serde(rename = "vBCFCPST")]
    pub v_bcfcpst: Option<String>,
    #[serde(rename = "pFCPST")]
    pub p_fcpst: Option<String>,
    #[serde(rename = "vFCPST")]
    pub v_fcpst: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Icms20 {
    #[serde(rename = "orig")]
    pub orig: Option<String>,
    #[serde(rename = "CST")]
    pub cst: Option<String>,
    #[serde(rename = "modBC")]
    pub mod_bc: Option<String>,
    #[serde(rename = "pRedBC")]
    pub p_red_bc: Option<String>,
    #[serde(rename = "vBC")]
    pub v_bc: Option<String>,
    #[serde(rename = "pICMS")]
    pub p_icms: Option<String>,
    #[serde(rename = "vICMS")]
    pub v_icms: Option<String>,
    #[serde(rename = "vBCFCP")]
    pub v_bcfcp: Option<String>,
    #[serde(rename = "pFCP")]
    pub p_fcp: Option<String>,
    #[serde(rename = "vFCP")]
    pub v_fcp: Option<String>,
    #[serde(rename = "vICMSDeson")]
    pub v_icmsdeson: Option<String>,
    #[serde(rename = "motDesICMS")]
    pub mot_des_icms: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Icms30 {
    #[serde(rename = "orig")]
    pub orig: Option<String>,
    #[serde(rename = "CST")]
    pub cst: Option<String>,
    #[serde(rename = "modBCST")]
    pub mod_bcst: Option<String>,
    #[serde(rename = "pMVAST")]
    pub p_mvast: Option<String>,
    #[serde(rename = "pRedBCST")]
    pub p_red_bcst: Option<String>,
    #[serde(rename = "vBCST")]
    pub v_bcst: Option<String>,
    #[serde(rename = "pICMSST")]
    pub p_icmsst: Option<String>,
    #[serde(rename = "vICMSST")]
    pub v_icmsst: Option<String>,
    #[serde(rename = "vBCFCPST")]
    pub v_bcfcpst: Option<String>,
    #[serde(rename = "pFCPST")]
    pub p_fcpst: Option<String>,
    #[serde(rename = "vFCPST")]
    pub v_fcpst: Option<String>,
    #[serde(rename = "vICMSDeson")]
    pub v_icmsdeson: Option<String>,
    #[serde(rename = "motDesICMS")]
    pub mot_des_icms: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Icms40 {
    #[serde(rename = "orig")]
    pub orig: Option<String>,
    #[serde(rename = "CST")]
    pub cst: Option<String>,
    #[serde(rename = "vICMSDeson")]
    pub v_icmsdeson: Option<String>,
    #[serde(rename = "motDesICMS")]
    pub mot_des_icms: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Icms51 {
    #[serde(rename = "orig")]
    pub orig: Option<String>,
    #[serde(rename = "CST")]
    pub cst: Option<String>,
    #[serde(rename = "modBC")]
    pub mod_bc: Option<String>,
    #[serde(rename = "pRedBC")]
    pub p_red_bc: Option<String>,
    #[serde(rename = "vBC")]
    pub v_bc: Option<String>,
    #[serde(rename = "pICMS")]
    pub p_icms: Option<String>,
    #[serde(rename = "vICMSOp")]
    pub v_icmsop: Option<String>,
    #[serde(rename = "pDif")]
    pub p_dif: Option<String>,
    #[serde(rename = "vICMSDif")]
    pub v_icmsdif: Option<String>,
    #[serde(rename = "vICMS")]
    pub v_icms: Option<String>,
    #[serde(rename = "vBCFCP")]
    pub v_bcfcp: Option<String>,
    #[serde(rename = "pFCP")]
    pub p_fcp: Option<String>,
    #[serde(rename = "vFCP")]
    pub v_fcp: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Icms60 {
    #[serde(rename = "orig")]
    pub orig: Option<String>,
    #[serde(rename = "CST")]
    pub cst: Option<String>,
    #[serde(rename = "vBCSTRet")]
    pub v_bcstret: Option<String>,
    #[serde(rename = "pST")]
    pub p_st: Option<String>,
    #[serde(rename = "vICMSSubstituto")]
    pub v_icmssubstituto: Option<String>,
    #[serde(rename = "vICMSSTRet")]
    pub v_icmsstret: Option<String>,
    #[serde(rename = "vBCFCPSTRet")]
    pub v_bcfcpstret: Option<String>,
    #[serde(rename = "pFCPSTRet")]
    pub p_fcpstret: Option<String>,
    #[serde(rename = "vFCPSTRet")]
    pub v_fcpstret: Option<String>,
    #[serde(rename = "pRedBCEfet")]
    pub p_red_bcefet: Option<String>,
    #[serde(rename = "vBCEfet")]
    pub v_bcefet: Option<String>,
    #[serde(rename = "pICMSEfet")]
    pub p_icmsefet: Option<String>,
    #[serde(rename = "vICMSEfet")]
    pub v_icmsefet: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Icms70 {
    #[serde(rename = "orig")]
    pub orig: Option<String>,
    #[serde(rename = "CST")]
    pub cst: Option<String>,
    #[serde(rename = "modBC")]
    pub mod_bc: Option<String>,
    #[serde(rename = "pRedBC")]
    pub p_red_bc: Option<String>,
    #[serde(rename = "vBC")]
    pub v_bc: Option<String>,
    #[serde(rename = "pICMS")]
    pub p_icms: Option<String>,
    #[serde(rename = "vICMS")]
    pub v_icms: Option<String>,
    #[serde(rename = "vBCFCP")]
    pub v_bcfcp: Option<String>,
    #[serde(rename = "pFCP")]
    pub p_fcp: Option<String>,
    #[serde(rename = "vFCP")]
    pub v_fcp: Option<String>,
    #[serde(rename = "modBCST")]
    pub mod_bcst: Option<String>,
    #[serde(rename = "pMVAST")]
    pub p_mvast: Option<String>,
    #[serde(rename = "pRedBCST")]
    pub p_red_bcst: Option<String>,
    #[serde(rename = "vBCST")]
    pub v_bcst: Option<String>,
    #[serde(rename = "pICMSST")]
    pub p_icmsst: Option<String>,
    #[serde(rename = "vICMSST")]
    pub v_icmsst: Option<String>,
    #[serde(rename = "vBCFCPST")]
    pub v_bcfcpst: Option<String>,
    #[serde(rename = "pFCPST")]
    pub p_fcpst: Option<String>,
    #[serde(rename = "vFCPST")]
    pub v_fcpst: Option<String>,
    #[serde(rename = "vICMSDeson")]
    pub v_icmsdeson: Option<String>,
    #[serde(rename = "motDesICMS")]
    pub mot_des_icms: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Icms90 {
    #[serde(rename = "orig")]
    pub orig: Option<String>,
    #[serde(rename = "CST")]
    pub cst: Option<String>,
    #[serde(rename = "modBC")]
    pub mod_bc: Option<String>,
    #[serde(rename = "vBC")]
    pub v_bc: Option<String>,
    #[serde(rename = "pRedBC")]
    pub p_red_bc: Option<String>,
    #[serde(rename = "pICMS")]
    pub p_icms: Option<String>,
    #[serde(rename = "vICMS")]
    pub v_icms: Option<String>,
    #[serde(rename = "vBCFCP")]
    pub v_bcfcp: Option<String>,
    #[serde(rename = "pFCP")]
    pub p_fcp: Option<String>,
    #[serde(rename = "vFCP")]
    pub v_fcp: Option<String>,
    #[serde(rename = "modBCST")]
    pub mod_bcst: Option<String>,
    #[serde(rename = "pMVAST")]
    pub p_mvast: Option<String>,
    #[serde(rename = "pRedBCST")]
    pub p_red_bcst: Option<String>,
    #[serde(rename = "vBCST")]
    pub v_bcst: Option<String>,
    #[serde(rename = "pICMSST")]
    pub p_icmsst: Option<String>,
    #[serde(rename = "vICMSST")]
    pub v_icmsst: Option<String>,
    #[serde(rename = "vBCFCPST")]
    pub v_bcfcpst: Option<String>,
    #[serde(rename = "pFCPST")]
    pub p_fcpst: Option<String>,
    #[serde(rename = "vFCPST")]
    pub v_fcpst: Option<String>,
    #[serde(rename = "vICMSDeson")]
    pub v_icmsdeson: Option<String>,
    #[serde(rename = "motDesICMS")]
    pub mot_des_icms: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IcmsoutraUf {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "CST")]
    pub cst: String,
    #[serde(rename = "pICMSOutraUF")]
    pub p_icmsoutra_uf: String,
    #[serde(rename = "pRedBCOutraUF")]
    pub p_red_bcoutra_uf: String,
    #[serde(rename = "vBCOutraUF")]
    pub v_bcoutra_uf: String,
    #[serde(rename = "vICMSOutraUF")]
    pub v_icmsoutra_uf: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Icmssn {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "CST")]
    pub cst: String,
    #[serde(rename = "indSN")]
    pub ind_sn: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Icmspart {
    #[serde(rename = "orig")]
    pub orig: Option<String>,
    #[serde(rename = "CST")]
    pub cst: Option<String>,
    #[serde(rename = "modBC")]
    pub mod_bc: Option<String>,
    #[serde(rename = "vBC")]
    pub v_bc: Option<String>,
    #[serde(rename = "pRedBC")]
    pub p_red_bc: Option<String>,
    #[serde(rename = "pICMS")]
    pub p_icms: Option<String>,
    #[serde(rename = "vICMS")]
    pub v_icms: Option<String>,
    #[serde(rename = "modBCST")]
    pub mod_bcst: Option<String>,
    #[serde(rename = "pMVAST")]
    pub p_mvast: Option<String>,
    #[serde(rename = "pRedBCST")]
    pub p_red_bcst: Option<String>,
    #[serde(rename = "vBCST")]
    pub v_bcst: Option<String>,
    #[serde(rename = "pICMSST")]
    pub p_icmsst: Option<String>,
    #[serde(rename = "vICMSST")]
    pub v_icmsst: Option<String>,
    #[serde(rename = "pBCOp")]
    pub p_bcop: Option<String>,
    #[serde(rename = "UFST")]
    pub ufst: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Icmsst {
    #[serde(rename = "orig")]
    pub orig: Option<String>,
    #[serde(rename = "CST")]
    pub cst: Option<String>,
    #[serde(rename = "vBCSTRet")]
    pub v_bcstret: Option<String>,
    #[serde(rename = "pST")]
    pub p_st: Option<String>,
    #[serde(rename = "vICMSSubstituto")]
    pub v_icmssubstituto: Option<String>,
    #[serde(rename = "vICMSSTRet")]
    pub v_icmsstret: Option<String>,
    #[serde(rename = "vBCFCPSTRet")]
    pub v_bcfcpstret: Option<String>,
    #[serde(rename = "pFCPSTRet")]
    pub p_fcpstret: Option<String>,
    #[serde(rename = "vFCPSTRet")]
    pub v_fcpstret: Option<String>,
    #[serde(rename = "vBCSTDest")]
    pub v_bcstdest: Option<String>,
    #[serde(rename = "vICMSSTDest")]
    pub v_icmsstdest: Option<String>,
    #[serde(rename = "pRedBCEfet")]
    pub p_red_bcefet: Option<String>,
    #[serde(rename = "vBCEfet")]
    pub v_bcefet: Option<String>,
    #[serde(rename = "pICMSEfet")]
    pub p_icmsefet: Option<String>,
    #[serde(rename = "vICMSEfet")]
    pub v_icmsefet: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Icmssn101 {
    #[serde(rename = "orig")]
    pub orig: Option<String>,
    #[serde(rename = "CSOSN")]
    pub csosn: Option<String>,
    #[serde(rename = "pCredSN")]
    pub p_cred_sn: Option<String>,
    #[serde(rename = "vCredICMSSN")]
    pub v_cred_icmssn: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Icmssn102 {
    #[serde(rename = "orig")]
    pub orig: Option<String>,
    #[serde(rename = "CSOSN")]
    pub csosn: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Icmssn201 {
    #[serde(rename = "orig")]
    pub orig: Option<String>,
    #[serde(rename = "CSOSN")]
    pub csosn: Option<String>,
    #[serde(rename = "modBCST")]
    pub mod_bcst: Option<String>,
    #[serde(rename = "pMVAST")]
    pub p_mvast: Option<String>,
    #[serde(rename = "pRedBCST")]
    pub p_red_bcst: Option<String>,
    #[serde(rename = "vBCST")]
    pub v_bcst: Option<String>,
    #[serde(rename = "pICMSST")]
    pub p_icmsst: Option<String>,
    #[serde(rename = "vICMSST")]
    pub v_icmsst: Option<String>,
    #[serde(rename = "vBCFCPST")]
    pub v_bcfcpst: Option<String>,
    #[serde(rename = "pFCPST")]
    pub p_fcpst: Option<String>,
    #[serde(rename = "vFCPST")]
    pub v_fcpst: Option<String>,
    #[serde(rename = "pCredSN")]
    pub p_cred_sn: Option<String>,
    #[serde(rename = "vCredICMSSN")]
    pub v_cred_icmssn: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Icmssn202 {
    #[serde(rename = "orig")]
    pub orig: Option<String>,
    #[serde(rename = "CSOSN")]
    pub csosn: Option<String>,
    #[serde(rename = "modBCST")]
    pub mod_bcst: Option<String>,
    #[serde(rename = "pMVAST")]
    pub p_mvast: Option<String>,
    #[serde(rename = "pRedBCST")]
    pub p_red_bcst: Option<String>,
    #[serde(rename = "vBCST")]
    pub v_bcst: Option<String>,
    #[serde(rename = "pICMSST")]
    pub p_icmsst: Option<String>,
    #[serde(rename = "vICMSST")]
    pub v_icmsst: Option<String>,
    #[serde(rename = "vBCFCPST")]
    pub v_bcfcpst: Option<String>,
    #[serde(rename = "pFCPST")]
    pub p_fcpst: Option<String>,
    #[serde(rename = "vFCPST")]
    pub v_fcpst: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Icmssn500 {
    #[serde(rename = "orig")]
    pub orig: Option<String>,
    #[serde(rename = "CSOSN")]
    pub csosn: Option<String>,
    #[serde(rename = "vBCSTRet")]
    pub v_bcstret: Option<String>,
    #[serde(rename = "pST")]
    pub p_st: Option<String>,
    #[serde(rename = "vICMSSubstituto")]
    pub v_icmssubstituto: Option<String>,
    #[serde(rename = "vICMSSTRet")]
    pub v_icmsstret: Option<String>,
    #[serde(rename = "vBCFCPSTRet")]
    pub v_bcfcpstret: Option<String>,
    #[serde(rename = "pFCPSTRet")]
    pub p_fcpstret: Option<String>,
    #[serde(rename = "vFCPSTRet")]
    pub v_fcpstret: Option<String>,
    #[serde(rename = "pRedBCEfet")]
    pub p_red_bcefet: Option<String>,
    #[serde(rename = "vBCEfet")]
    pub v_bcefet: Option<String>,
    #[serde(rename = "pICMSEfet")]
    pub p_icmsefet: Option<String>,
    #[serde(rename = "vICMSEfet")]
    pub v_icmsefet: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Icmssn900 {
    #[serde(rename = "orig")]
    pub orig: Option<String>,
    #[serde(rename = "CSOSN")]
    pub csosn: Option<String>,
    #[serde(rename = "modBC")]
    pub mod_bc: Option<String>,
    #[serde(rename = "vBC")]
    pub v_bc: Option<String>,
    #[serde(rename = "pRedBC")]
    pub p_red_bc: Option<String>,
    #[serde(rename = "pICMS")]
    pub p_icms: Option<String>,
    #[serde(rename = "vICMS")]
    pub v_icms: Option<String>,
    #[serde(rename = "modBCST")]
    pub mod_bcst: Option<String>,
    #[serde(rename = "pMVAST")]
    pub p_mvast: Option<String>,
    #[serde(rename = "pRedBCST")]
    pub p_red_bcst: Option<String>,
    #[serde(rename = "vBCST")]
    pub v_bcst: Option<String>,
    #[serde(rename = "pICMSST")]
    pub p_icmsst: Option<String>,
    #[serde(rename = "vICMSST")]
    pub v_icmsst: Option<String>,
    #[serde(rename = "vBCFCPST")]
    pub v_bcfcpst: Option<String>,
    #[serde(rename = "pFCPST")]
    pub p_fcpst: Option<String>,
    #[serde(rename = "vFCPST")]
    pub v_fcpst: Option<String>,
    #[serde(rename = "pCredSN")]
    pub p_cred_sn: Option<String>,
    #[serde(rename = "vCredICMSSN")]
    pub v_cred_icmssn: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Icmsuffim {
    #[serde(rename = "vBCUFFim")]
    pub v_bcuffim: Option<String>,
    #[serde(rename = "pFCPUFFim")]
    pub p_fcpuffim: Option<String>,
    #[serde(rename = "pICMSUFFim")]
    pub p_icmsuffim: Option<String>,
    #[serde(rename = "pICMSInter")]
    pub p_icmsinter: Option<String>,
    #[serde(rename = "vFCPUFFim")]
    pub v_fcpuffim: Option<String>,
    #[serde(rename = "vICMSUFFim")]
    pub v_icmsuffim: Option<String>,
    #[serde(rename = "vICMSUFIni")]
    pub v_icmsufini: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "pICMSInterPart")]
    pub p_icmsinter_part: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Ii {
    #[serde(rename = "vBC")]
    pub v_bc: Option<String>,
    #[serde(rename = "vDespAdu")]
    pub v_desp_adu: Option<String>,
    #[serde(rename = "vII")]
    pub v_ii: Option<String>,
    #[serde(rename = "vIOF")]
    pub v_iof: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Ipi {
    #[serde(rename = "CNPJProd")]
    pub cnpjprod: Option<String>,
    #[serde(rename = "cSelo")]
    pub c_selo: Option<String>,
    #[serde(rename = "qSelo")]
    pub q_selo: Option<String>,
    #[serde(rename = "cEnq")]
    pub c_enq: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "IPITrib")]
    pub ipitrib: Option<Ipitrib>,
    #[serde(rename = "IPINT")]
    pub ipint: Option<Ipint>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Ipitrib {
    #[serde(rename = "CST")]
    pub cst: Option<String>,
    #[serde(rename = "vBC")]
    pub v_bc: Option<String>,
    #[serde(rename = "pIPI")]
    pub p_ipi: Option<String>,
    #[serde(rename = "qUnid")]
    pub q_unid: Option<String>,
    #[serde(rename = "vUnid")]
    pub v_unid: Option<String>,
    #[serde(rename = "vIPI")]
    pub v_ipi: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Ipint {
    #[serde(rename = "CST")]
    pub cst: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Issqn {
    #[serde(rename = "vBC")]
    pub v_bc: Option<String>,
    #[serde(rename = "vAliq")]
    pub v_aliq: Option<String>,
    #[serde(rename = "vISSQN")]
    pub v_issqn: Option<String>,
    #[serde(rename = "cMunFG")]
    pub c_mun_fg: Option<String>,
    #[serde(rename = "cListServ")]
    pub c_list_serv: Option<String>,
    #[serde(rename = "vDeducao")]
    pub v_deducao: Option<String>,
    #[serde(rename = "vOutro")]
    pub v_outro: Option<String>,
    #[serde(rename = "vDescIncond")]
    pub v_desc_incond: Option<String>,
    #[serde(rename = "vDescCond")]
    pub v_desc_cond: Option<String>,
    #[serde(rename = "vISSRet")]
    pub v_issret: Option<String>,
    #[serde(rename = "indISS")]
    pub ind_iss: Option<String>,
    #[serde(rename = "cServico")]
    pub c_servico: Option<String>,
    #[serde(rename = "cMun")]
    pub c_mun: Option<String>,
    #[serde(rename = "cPais")]
    pub c_pais: Option<String>,
    #[serde(rename = "nProcesso")]
    pub n_processo: Option<String>,
    #[serde(rename = "indIncentivo")]
    pub ind_incentivo: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Pis {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "PISAliq")]
    pub pisaliq: Option<Pisaliq>,
    #[serde(rename = "PISQtde")]
    pub pisqtde: Option<Pisqtde>,
    #[serde(rename = "PISNT")]
    pub pisnt: Option<Pisnt>,
    #[serde(rename = "PISOutr")]
    pub pisoutr: Option<Pisoutr>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Pisaliq {
    #[serde(rename = "CST")]
    pub cst: Option<String>,
    #[serde(rename = "vBC")]
    pub v_bc: Option<String>,
    #[serde(rename = "pPIS")]
    pub p_pis: Option<String>,
    #[serde(rename = "vPIS")]
    pub v_pis: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Pisqtde {
    #[serde(rename = "CST")]
    pub cst: Option<String>,
    #[serde(rename = "qBCProd")]
    pub q_bcprod: Option<String>,
    #[serde(rename = "vAliqProd")]
    pub v_aliq_prod: Option<String>,
    #[serde(rename = "vPIS")]
    pub v_pis: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Pisnt {
    #[serde(rename = "CST")]
    pub cst: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Pisoutr {
    #[serde(rename = "CST")]
    pub cst: Option<String>,
    #[serde(rename = "vBC")]
    pub v_bc: Option<String>,
    #[serde(rename = "pPIS")]
    pub p_pis: Option<String>,
    #[serde(rename = "qBCProd")]
    pub q_bcprod: Option<String>,
    #[serde(rename = "vAliqProd")]
    pub v_aliq_prod: Option<String>,
    #[serde(rename = "vPIS")]
    pub v_pis: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Pisst {
    #[serde(rename = "vBC")]
    pub v_bc: Option<String>,
    #[serde(rename = "pPIS")]
    pub p_pis: Option<String>,
    #[serde(rename = "qBCProd")]
    pub q_bcprod: Option<String>,
    #[serde(rename = "vAliqProd")]
    pub v_aliq_prod: Option<String>,
    #[serde(rename = "vPIS")]
    pub v_pis: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Cofins {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "COFINSAliq")]
    pub cofinsaliq: Option<Cofinsaliq>,
    #[serde(rename = "COFINSQtde")]
    pub cofinsqtde: Option<Cofinsqtde>,
    #[serde(rename = "COFINSNT")]
    pub cofinsnt: Option<Cofinsnt>,
    #[serde(rename = "COFINSOutr")]
    pub cofinsoutr: Option<Cofinsoutr>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Cofinsaliq {
    #[serde(rename = "CST")]
    pub cst: Option<String>,
    #[serde(rename = "vBC")]
    pub v_bc: Option<String>,
    #[serde(rename = "pCOFINS")]
    pub p_cofins: Option<String>,
    #[serde(rename = "vCOFINS")]
    pub v_cofins: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Cofinsqtde {
    #[serde(rename = "CST")]
    pub cst: Option<String>,
    #[serde(rename = "qBCProd")]
    pub q_bcprod: Option<String>,
    #[serde(rename = "vAliqProd")]
    pub v_aliq_prod: Option<String>,
    #[serde(rename = "vCOFINS")]
    pub v_cofins: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Cofinsnt {
    #[serde(rename = "CST")]
    pub cst: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Cofinsoutr {
    #[serde(rename = "CST")]
    pub cst: Option<String>,
    #[serde(rename = "vBC")]
    pub v_bc: Option<String>,
    #[serde(rename = "pCOFINS")]
    pub p_cofins: Option<String>,
    #[serde(rename = "qBCProd")]
    pub q_bcprod: Option<String>,
    #[serde(rename = "vAliqProd")]
    pub v_aliq_prod: Option<String>,
    #[serde(rename = "vCOFINS")]
    pub v_cofins: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Cofinsst {
    #[serde(rename = "vBC")]
    pub v_bc: Option<String>,
    #[serde(rename = "pCOFINS")]
    pub p_cofins: Option<String>,
    #[serde(rename = "qBCProd")]
    pub q_bcprod: Option<String>,
    #[serde(rename = "vAliqProd")]
    pub v_aliq_prod: Option<String>,
    #[serde(rename = "vCOFINS")]
    pub v_cofins: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Icmsufdest {
    #[serde(rename = "vBCUFDest")]
    pub v_bcufdest: Option<String>,
    #[serde(rename = "vBCFCPUFDest")]
    pub v_bcfcpufdest: Option<String>,
    #[serde(rename = "pFCPUFDest")]
    pub p_fcpufdest: Option<String>,
    #[serde(rename = "pICMSUFDest")]
    pub p_icmsufdest: Option<String>,
    #[serde(rename = "pICMSInter")]
    pub p_icmsinter: Option<String>,
    #[serde(rename = "pICMSInterPart")]
    pub p_icmsinter_part: Option<String>,
    #[serde(rename = "vFCPUFDest")]
    pub v_fcpufdest: Option<String>,
    #[serde(rename = "vICMSUFDest")]
    pub v_icmsufdest: Option<String>,
    #[serde(rename = "vICMSUFRemet")]
    pub v_icmsufremet: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImpostoDevol {
    #[serde(rename = "pDevol")]
    pub p_devol: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "IPI")]
    pub ipi: Option<ImpostoDevolNfeIpi>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImpostoDevolNfeIpi {
    #[serde(rename = "vIPIDevol")]
    pub v_ipidevol: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Total {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "ICMSTot")]
    pub icmstot: Option<Icmstot>,
    #[serde(rename = "ISSQNtot")]
    pub issqntot: Option<Issqntot>,
    #[serde(rename = "retTrib")]
    pub ret_trib: Option<RetTrib>,
}

impl Total {
    pub fn get_valor_total_da_nota_fiscal(&self) -> Option<f64> {
        self
        .icmstot
        .as_ref()
        .and_then(|icms_total| {
            icms_total
                .v_nf
                .as_ref()
                .and_then(|v| v.trim().parse().ok())
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Icmstot {
    #[serde(rename = "vBC")]
    pub v_bc: Option<String>,
    #[serde(rename = "vICMS")]
    pub v_icms: Option<String>,
    #[serde(rename = "vICMSDeson")]
    pub v_icmsdeson: Option<String>,
    #[serde(rename = "vFCPUFDest")]
    pub v_fcpufdest: Option<String>,
    #[serde(rename = "vICMSUFDest")]
    pub v_icmsufdest: Option<String>,
    #[serde(rename = "vICMSUFRemet")]
    pub v_icmsufremet: Option<String>,
    #[serde(rename = "vFCP")]
    pub v_fcp: Option<String>,
    #[serde(rename = "vBCST")]
    pub v_bcst: Option<String>,
    #[serde(rename = "vST")]
    pub v_st: Option<String>,
    #[serde(rename = "vFCPST")]
    pub v_fcpst: Option<String>,
    #[serde(rename = "vFCPSTRet")]
    pub v_fcpstret: Option<String>,
    #[serde(rename = "vProd")]
    pub v_prod: Option<String>,
    #[serde(rename = "vFrete")]
    pub v_frete: Option<String>,
    #[serde(rename = "vSeg")]
    pub v_seg: Option<String>,
    #[serde(rename = "vDesc")]
    pub v_desc: Option<String>,
    #[serde(rename = "vII")]
    pub v_ii: Option<String>,
    #[serde(rename = "vIPI")]
    pub v_ipi: Option<String>,
    #[serde(rename = "vIPIDevol")]
    pub v_ipidevol: Option<String>,
    #[serde(rename = "vPIS")]
    pub v_pis: Option<String>,
    #[serde(rename = "vCOFINS")]
    pub v_cofins: Option<String>,
    #[serde(rename = "vOutro")]
    pub v_outro: Option<String>,
    #[serde(rename = "vNF")]
    pub v_nf: Option<String>,
    #[serde(rename = "vTotTrib")]
    pub v_tot_trib: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Issqntot {
    #[serde(rename = "vServ")]
    pub v_serv: Option<String>,
    #[serde(rename = "vBC")]
    pub v_bc: Option<String>,
    #[serde(rename = "vISS")]
    pub v_iss: Option<String>,
    #[serde(rename = "vPIS")]
    pub v_pis: Option<String>,
    #[serde(rename = "vCOFINS")]
    pub v_cofins: Option<String>,
    #[serde(rename = "dCompet")]
    pub d_compet: Option<String>,
    #[serde(rename = "vDeducao")]
    pub v_deducao: Option<String>,
    #[serde(rename = "vOutro")]
    pub v_outro: Option<String>,
    #[serde(rename = "vDescIncond")]
    pub v_desc_incond: Option<String>,
    #[serde(rename = "vDescCond")]
    pub v_desc_cond: Option<String>,
    #[serde(rename = "vISSRet")]
    pub v_issret: Option<String>,
    #[serde(rename = "cRegTrib")]
    pub c_reg_trib: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RetTrib {
    #[serde(rename = "vRetPIS")]
    pub v_ret_pis: Option<String>,
    #[serde(rename = "vRetCOFINS")]
    pub v_ret_cofins: Option<String>,
    #[serde(rename = "vRetCSLL")]
    pub v_ret_csll: Option<String>,
    #[serde(rename = "vBCIRRF")]
    pub v_bcirrf: Option<String>,
    #[serde(rename = "vIRRF")]
    pub v_irrf: Option<String>,
    #[serde(rename = "vBCRetPrev")]
    pub v_bcret_prev: Option<String>,
    #[serde(rename = "vRetPrev")]
    pub v_ret_prev: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}
use claudiofsr_lib::OptionExtension;
use serde::{Deserialize, Serialize};

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
    pub fn get_cst_pis(&self) -> Option<u8> {
        self.pis.as_ref().and_then(|p| p.get_pis_cst())
    }

    pub fn get_aliq_pis(&self) -> Option<f64> {
        self.pis.as_ref().and_then(|p| p.get_pis_aliquota())
    }

    pub fn get_v_pis(&self) -> Option<f64> {
        self.pis.as_ref().and_then(|p| p.get_pis_valor())
    }

    pub fn get_cst_cofins(&self) -> Option<u8> {
        self.cofins.as_ref().and_then(|c| c.get_cofins_cst())
    }

    pub fn get_aliq_cofins(&self) -> Option<f64> {
        self.cofins.as_ref().and_then(|c| c.get_cofins_aliquota())
    }

    pub fn get_v_cofins(&self) -> Option<f64> {
        self.cofins.as_ref().and_then(|c| c.get_cofins_valor())
    }

    pub fn get_v_bc_icms(&self) -> Option<f64> {
        self.icms
            .as_ref()
            .and_then(|imposto_de_circulacao| imposto_de_circulacao.get_icms_valor_base_calc())
    }

    pub fn get_aliq_icms(&self) -> Option<f64> {
        self.icms
            .as_ref()
            .and_then(|imposto_de_circulacao| imposto_de_circulacao.get_icms_aliquota())
    }

    pub fn get_v_icms(&self) -> Option<f64> {
        self.icms
            .as_ref()
            .and_then(|imposto_de_circulacao| imposto_de_circulacao.get_icms_valor_tributo())
    }

    pub fn get_v_iss(&self) -> Option<f64> {
        self.issqn
            .as_ref()
            .and_then(|imposto_sobre_servico| imposto_sobre_servico.get_valor_iss())
    }

    pub fn get_v_ipi(&self) -> Option<f64> {
        self.ipi.as_ref().and_then(|imposto_sobre_produto| {
            imposto_sobre_produto
                .iter()
                .map(|tributo| tributo.get_valor_ipi())
                .sum()
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

impl Icms {
    /// Get valor da Base de Cálculo do ICMS
    pub fn get_icms_valor_base_calc(&self) -> Option<f64> {
        let bc_icms00: Option<f64> = self.icms00.as_ref().and_then(|icms| icms.v_bc.parse());

        if bc_icms00.is_some() {
            return bc_icms00;
        }

        let bc_icms10: Option<f64> = self.icms10.as_ref().and_then(|icms| icms.v_bc.parse());

        if bc_icms10.is_some() {
            return bc_icms10;
        }

        let bc_icms20: Option<f64> = self.icms20.as_ref().and_then(|icms| icms.v_bc.parse());

        bc_icms20
    }

    /// Get Alíquota de ICMS
    pub fn get_icms_aliquota(&self) -> Option<f64> {
        let aliquota00: Option<f64> = self.icms00.as_ref().and_then(|icms| icms.p_icms.parse());

        if aliquota00.is_some() {
            return aliquota00;
        }

        let aliquota10: Option<f64> = self.icms10.as_ref().and_then(|icms| icms.p_icms.parse());

        if aliquota10.is_some() {
            return aliquota10;
        }

        let aliquota20: Option<f64> = self.icms20.as_ref().and_then(|icms| icms.p_icms.parse());

        aliquota20
    }

    /// Get valor do ICMS
    pub fn get_icms_valor_tributo(&self) -> Option<f64> {
        let bc_icms00: Option<f64> = self.icms00.as_ref().and_then(|icms| icms.v_icms.parse());

        if bc_icms00.is_some() {
            return bc_icms00;
        }

        let bc_icms10: Option<f64> = self.icms10.as_ref().and_then(|icms| icms.v_icms.parse());

        if bc_icms10.is_some() {
            return bc_icms10;
        }

        let bc_icms20: Option<f64> = self.icms20.as_ref().and_then(|icms| icms.v_icms.parse());

        bc_icms20
    }
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
    pub cst: Option<String>,
    #[serde(rename = "pICMSOutraUF")]
    pub p_icmsoutra_uf: Option<String>,
    #[serde(rename = "pRedBCOutraUF")]
    pub p_red_bcoutra_uf: Option<String>,
    #[serde(rename = "vBCOutraUF")]
    pub v_bcoutra_uf: Option<String>,
    #[serde(rename = "vICMSOutraUF")]
    pub v_icmsoutra_uf: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Icmssn {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "CST")]
    pub cst: Option<String>,
    #[serde(rename = "indSN")]
    pub ind_sn: Option<String>,
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

impl Ipi {
    fn get_valor_ipi(&self) -> Option<f64> {
        self.ipitrib.as_ref().and_then(|trib| trib.v_ipi.parse())
    }
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

impl Issqn {
    fn get_valor_iss(&self) -> Option<f64> {
        self.v_issqn.parse()
    }
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

impl Pis {
    /// Get CST de PIS
    pub fn get_pis_cst(&self) -> Option<u8> {
        let aliquota: Option<u8> = self
            .pisaliq
            .as_ref()
            .and_then(|pis_aliquota| pis_aliquota.cst.parse());

        if aliquota.is_some() {
            return aliquota;
        }

        let quantidade: Option<u8> = self
            .pisqtde
            .as_ref()
            .and_then(|pis_quantidade| pis_quantidade.cst.parse());

        if quantidade.is_some() {
            return quantidade;
        }

        let nao_tributado: Option<u8> = self.pisnt.as_ref().and_then(|pis_nt| pis_nt.cst.parse());

        if nao_tributado.is_some() {
            return nao_tributado;
        }

        let outros: Option<u8> = self
            .pisoutr
            .as_ref()
            .and_then(|pis_outros| pis_outros.cst.parse());

        outros
    }

    /// Get Alíquota de PIS
    pub fn get_pis_aliquota(&self) -> Option<f64> {
        let aliquota: Option<f64> = self
            .pisaliq
            .as_ref()
            .and_then(|pis_aliquota| pis_aliquota.p_pis.parse());

        if aliquota.is_some() {
            return aliquota;
        }

        let quantidade: Option<f64> = self
            .pisqtde
            .as_ref()
            .and_then(|pis_quantidade| pis_quantidade.v_aliq_prod.parse());

        if quantidade.is_some() {
            return quantidade;
        }

        let outros: Option<f64> = self
            .pisoutr
            .as_ref()
            .and_then(|pis_outros| pis_outros.p_pis.parse());

        outros
    }

    /// Get Valor de PIS
    pub fn get_pis_valor(&self) -> Option<f64> {
        let aliquota: Option<f64> = self
            .pisaliq
            .as_ref()
            .and_then(|pis_aliquota| pis_aliquota.v_pis.parse());

        if aliquota.is_some() {
            return aliquota;
        }

        let quantidade: Option<f64> = self
            .pisqtde
            .as_ref()
            .and_then(|pis_quantidade| pis_quantidade.v_aliq_prod.parse());

        if quantidade.is_some() {
            return quantidade;
        }

        let outros: Option<f64> = self
            .pisoutr
            .as_ref()
            .and_then(|pis_outros| pis_outros.v_pis.parse());

        outros
    }
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

impl Cofins {
    /// Get CST de COFINS
    pub fn get_cofins_cst(&self) -> Option<u8> {
        let aliquota: Option<u8> = self
            .cofinsaliq
            .as_ref()
            .and_then(|cofins_aliquota| cofins_aliquota.cst.parse());

        if aliquota.is_some() {
            return aliquota;
        }

        let quantidade: Option<u8> = self
            .cofinsqtde
            .as_ref()
            .and_then(|cofins_quantidade| cofins_quantidade.cst.parse());

        if quantidade.is_some() {
            return quantidade;
        }

        let nao_tributado: Option<u8> = self
            .cofinsnt
            .as_ref()
            .and_then(|cofins_nt| cofins_nt.cst.parse());

        if nao_tributado.is_some() {
            return nao_tributado;
        }

        let outros: Option<u8> = self
            .cofinsoutr
            .as_ref()
            .and_then(|cofins_outros| cofins_outros.cst.parse());

        outros
    }

    /// Get Alíquota de COFINS
    pub fn get_cofins_aliquota(&self) -> Option<f64> {
        let aliquota: Option<f64> = self
            .cofinsaliq
            .as_ref()
            .and_then(|cofins_aliquota| cofins_aliquota.p_cofins.parse());

        if aliquota.is_some() {
            return aliquota;
        }

        let quantidade: Option<f64> = self
            .cofinsqtde
            .as_ref()
            .and_then(|cofins_quantidade| cofins_quantidade.v_aliq_prod.parse());

        if quantidade.is_some() {
            return quantidade;
        }

        let outros: Option<f64> = self
            .cofinsoutr
            .as_ref()
            .and_then(|cofins_outros| cofins_outros.p_cofins.parse());

        outros
    }

    /// Get Valor de COFINS
    pub fn get_cofins_valor(&self) -> Option<f64> {
        let aliquota: Option<f64> = self
            .cofinsaliq
            .as_ref()
            .and_then(|cofins_aliquota| cofins_aliquota.v_cofins.parse());

        if aliquota.is_some() {
            return aliquota;
        }

        let quantidade: Option<f64> = self
            .cofinsqtde
            .as_ref()
            .and_then(|cofins_quantidade| cofins_quantidade.v_aliq_prod.parse());

        if quantidade.is_some() {
            return quantidade;
        }

        let outros: Option<f64> = self
            .cofinsoutr
            .as_ref()
            .and_then(|cofins_outros| cofins_outros.v_cofins.parse());

        outros
    }
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
    /// Valor Total da Nota Fiscal
    pub fn get_valor_nfe(&self) -> Option<f64> {
        self.icmstot
            .as_ref()
            .and_then(|icms_total| icms_total.v_nf.parse())
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

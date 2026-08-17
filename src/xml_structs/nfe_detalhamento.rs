//! # Estruturas de Detalhamento e Auxiliares da NF-e
//!
//! Este módulo gerencia os nós filhos e as estruturas acessórias aninhadas do XML da
//! Nota Fiscal Eletrônica (NF-e). Inclui o mapeamento de produtos, tributação setorial,
//! informações de rastreabilidade de lote, dados de transporte, cobrança,
//! intermediação comercial e informações adicionais do fisco ou contribuinte.
//!
//! Todas as estruturas adotam o zoneamento Serde com o atributo `default`
//! para assegurar compatibilidade posicional de leitura em fluxo e tolerância
//! a variações regulatórias do leiaute nacional.

use claudiofsr_lib::StrExtension;
use serde::{Deserialize, Serialize};

use crate::xml_structs::{aut_xml::InfProtocolo, impostos::Imposto};

/// Bloco de Detalhes dos Itens da NF-e (`<det>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Detalhes {
    /// Informações adicionais vinculadas ao produto deste item específico.
    #[serde(rename = "infAdProd", default)]
    pub inf_ad_prod: Option<String>,

    /// Número sequencial do item.
    #[serde(rename = "@nItem")]
    pub n_item: String,

    /// Conteúdo textual do nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    /// Dados descritivos do produto.
    #[serde(rename = "prod")]
    pub prod: Produto,

    /// Dados correspondentes ao bloco de impostos incidentes sobre o produto.
    #[serde(rename = "imposto")]
    pub imposto: Imposto,

    /// Dados correspondentes ao imposto devolvido se aplicável.
    #[serde(rename = "impostoDevol", default)]
    pub imposto_devol: Option<ImpostoDevol>,

    /// Valor líquido do item (valor do produto - descontos + acréscimos) (vItem).
    #[serde(rename = "vItem", default)]
    pub v_item: Option<String>,
}

/// Bloco de Informações dos Produtos e Serviços (`<prod>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Produto {
    /// Código atribuído internamente ao produto.
    #[serde(rename = "cProd", default)]
    pub c_prod: Option<String>,

    /// Código de barras global do produto (GTIN/EAN).
    #[serde(rename = "cEAN", default)]
    pub c_ean: Option<String>,

    /// Nome detalhado do produto.
    #[serde(rename = "xProd", default)]
    pub x_prod: Option<String>,

    /// Código NCM do item.
    #[serde(rename = "NCM", default)]
    pub ncm: Option<String>,

    /// Lista de codificações NVE associadas.
    #[serde(rename = "NVE", default)]
    pub nve: Option<Vec<String>>,

    /// Código CEST do item.
    #[serde(rename = "CEST", default)]
    pub cest: Option<String>,

    /// Indicador de produção em escala relevante.
    #[serde(rename = "indEscala", default)]
    pub ind_escala: Option<String>,

    /// CNPJ do Fabricante da mercadoria se aplicável.
    #[serde(rename = "CNPJFab", default)]
    pub cnpjfab: Option<String>,

    /// Código de benefício fiscal relacionado.
    #[serde(rename = "cBenef", default)]
    pub c_benef: Option<String>,

    /// Código de exceção da TIPI se aplicável.
    #[serde(rename = "EXTIPI", default)]
    pub extipi: Option<String>,

    /// CFOP correspondente ao item.
    #[serde(rename = "CFOP", default)]
    pub cfop: Option<String>,

    /// Unidade comercial de medida adotada.
    #[serde(rename = "uCom", default)]
    pub u_com: Option<String>,

    /// Quantidade comercial faturada.
    #[serde(rename = "qCom", default)]
    pub q_com: Option<String>,

    /// Valor bruto unitário comercializado.
    #[serde(rename = "vUnCom", default)]
    pub v_un_com: Option<String>,

    /// Valor bruto total do item (vProd).
    #[serde(rename = "vProd", default)]
    pub v_prod: Option<f64>,

    /// Código EAN de tributação correspondente.
    #[serde(rename = "cEANTrib", default)]
    pub c_eantrib: Option<String>,

    /// Unidade tributada de medida correspondente.
    #[serde(rename = "uTrib", default)]
    pub u_trib: Option<String>,

    /// Quantidade de produto tributada.
    #[serde(rename = "qTrib", default)]
    pub q_trib: Option<String>,

    /// Valor tributado unitário.
    #[serde(rename = "vUnTrib", default)]
    pub v_un_trib: Option<String>,

    /// Valor proporcional do frete.
    #[serde(rename = "vFrete", default)]
    pub v_frete: Option<f64>,

    /// Valor proporcional do seguro.
    #[serde(rename = "vSeg", default)]
    pub v_seg: Option<f64>,

    /// Valor total de descontos concedidos ao item.
    #[serde(rename = "vDesc", default)]
    pub v_desc: Option<f64>,

    /// Outros custos acessórios incidentes.
    #[serde(rename = "vOutro", default)]
    pub v_outro: Option<String>,

    /// Indicador se o valor do item entra na composição final do vNF.
    #[serde(rename = "indTot", default)]
    pub ind_tot: Option<String>,

    /// Número identificador do Pedido de Compra.
    #[serde(rename = "xPed", default)]
    pub x_ped: Option<String>,

    /// Número identificador do item do pedido correspondente.
    #[serde(rename = "nItemPed", default)]
    pub n_item_ped: Option<String>,

    /// Número de controle de importação (FCI) do produto.
    #[serde(rename = "nFCI", default)]
    pub n_fci: Option<String>,

    /// Número do Registro de Controle do Papel Imune (RECOPI).
    #[serde(rename = "nRECOPI", default)]
    pub n_recopi: Option<String>,

    /// Texto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    /// Dados correspondentes à Declaração de Importação (DI).
    #[serde(rename = "DI", default)]
    pub di: Option<Di>,

    /// Detalhes adicionais relativos a processos de exportação.
    #[serde(rename = "detExport", default)]
    pub det_export: Option<DetExport>,

    /// Lista de rastreamento do lote de produção.
    #[serde(rename = "rastro", default)]
    pub rastro: Option<Vec<Rastro>>,

    /// Dados específicos de faturamento de veículos novos.
    #[serde(rename = "veicProd", default)]
    pub veic_prod: Option<VeicProd>,

    /// Dados específicos de faturamento de medicamentos e insumos farmacêuticos.
    #[serde(rename = "med", default)]
    pub med: Option<Med>,

    /// Dados de faturamento de armamentos controlados.
    #[serde(rename = "arma", default)]
    pub arma: Option<Arma>,

    /// Dados específicos para faturamento de combustíveis líquidos ou gasosos.
    #[serde(rename = "comb", default)]
    pub comb: Option<Comb>,
}

impl Produto {
    /// Retorna a descrição detalhada sanitizada em caixa alta.
    pub fn get_descricao(&self) -> Option<String> {
        self.x_prod
            .as_ref()
            .map(|descr| descr.trim().replace_multiple_whitespaces().to_uppercase())
    }

    /// Retorna o CFOP de forma puramente numérica.
    pub fn get_cfop(&self) -> Option<u16> {
        self.cfop
            .as_ref()
            .and_then(|c| c.remove_non_digits().parse().ok())
    }

    /// Retorna o código NCM formatado.
    pub fn get_ncm(&self) -> Option<String> {
        self.ncm
            .as_ref()
            .map(|n| n.remove_non_digits().format_ncm())
    }

    /// Retorna de forma numérica o documento de importação se aplicável.
    pub fn get_num_dec_importacao(&self) -> Option<u64> {
        self.di
            .as_ref()
            .and_then(|dec_importacao| dec_importacao.get_num_di())
    }
}

/// Dados da Declaração de Importação (`<DI>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Di {
    /// Número identificador do documento de importação (DI/DSI/DA).
    #[serde(rename = "nDI", default)]
    pub n_di: Option<String>,

    /// Data de registro do documento de importação.
    #[serde(rename = "dDI", default)]
    pub d_di: Option<String>,

    /// Local onde ocorreu o desembaraço aduaneiro.
    #[serde(rename = "xLocDesemb", default)]
    pub x_loc_desemb: Option<String>,

    /// Unidade Federativa onde ocorreu o desembaraço.
    #[serde(rename = "UFDesemb", default)]
    pub ufdesemb: Option<String>,

    /// Data de efetivação do desembaraço aduaneiro.
    #[serde(rename = "dDesemb", default)]
    pub d_desemb: Option<String>,

    /// Tipo de via de transporte internacional utilizada.
    #[serde(rename = "tpViaTransp", default)]
    pub tp_via_transp: Option<String>,

    /// Valor monetário calculado de AFRMM se aplicável.
    #[serde(rename = "vAFRMM", default)]
    pub v_afrmm: Option<String>,

    /// Tipo de intermediação de importação realizada.
    #[serde(rename = "tpIntermedio", default)]
    pub tp_intermedio: Option<String>,

    /// CNPJ do adquirente ou encomendante se aplicável.
    #[serde(rename = "CNPJ", default)]
    pub cnpj: Option<String>,

    /// Unidade Federativa do adquirente.
    #[serde(rename = "UFTerceiro", default)]
    pub ufterceiro: Option<String>,

    /// Código identificador do exportador no exterior.
    #[serde(rename = "cExportador", default)]
    pub c_exportador: Option<String>,

    /// Texto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    /// Lista das adições correspondentes de importação.
    #[serde(rename = "adi", default)]
    pub adi: Option<Vec<Adi>>,
}

impl Di {
    /// Retorna o número identificador do documento de importação como valor numérico.
    fn get_num_di(&self) -> Option<u64> {
        self.n_di
            .as_ref()
            .and_then(|c| c.remove_non_digits().parse().ok())
    }
}

/// Informações de Adição de Importação (`<adi>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Adi {
    /// Número sequencial correspondente à Adição.
    #[serde(rename = "nAdicao", default)]
    pub n_adicao: Option<String>,

    /// Número sequencial do item correspondente à Adição.
    #[serde(rename = "nSeqAdic", default)]
    pub n_seq_adic: Option<String>,

    /// Código identificador internacional do fabricante do produto.
    #[serde(rename = "cFabricante", default)]
    pub c_fabricante: Option<String>,

    /// Desconto concedido na DI correspondente.
    #[serde(rename = "vDescDI", default)]
    pub v_desc_di: Option<String>,

    /// Número identificador correspondente ao processo Drawback.
    #[serde(rename = "nDraw", default)]
    pub n_draw: Option<String>,

    /// Texto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// Detalhes vinculados a processos de Exportação do item (`<detExport>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DetExport {
    /// Número identificador do processo Drawback vinculado.
    #[serde(rename = "nDraw", default)]
    pub n_draw: Option<String>,

    /// Texto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    /// Dados referentes à exportação indireta se aplicável.
    #[serde(rename = "exportInd", default)]
    pub export_ind: Option<ExportInd>,
}

/// Dados específicos sobre Exportação Indireta (`<exportInd>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExportInd {
    /// Número de Registro de Exportação emitido.
    #[serde(rename = "nRE", default)]
    pub n_re: Option<String>,

    /// Chave de acesso correspondente à NF-e recebida para fins de exportação.
    #[serde(rename = "chNFe", default)]
    pub ch_nfe: Option<String>,

    /// Quantidade de produto efetivamente exportada.
    #[serde(rename = "qExport", default)]
    pub q_export: Option<String>,

    /// Texto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// Dados do Rastreamento do Produto por Lotes (`<rastro>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Rastro {
    /// Número identificador atribuído ao Lote de produção.
    #[serde(rename = "nLote", default)]
    pub n_lote: Option<String>,

    /// Quantidade total de produto associada ao lote informado.
    #[serde(rename = "qLote", default)]
    pub q_lote: Option<String>,

    /// Data em que ocorreu a fabricação do produto.
    #[serde(rename = "dFab", default)]
    pub d_fab: Option<String>,

    /// Data limite de validade recomendada.
    #[serde(rename = "dVal", default)]
    pub d_val: Option<String>,

    /// Código de agregação associado se aplicável.
    #[serde(rename = "cAgreg", default)]
    pub c_agreg: Option<String>,

    /// Texto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// Dados específicos relativos ao faturamento de Veículos Novos (`<veicProd>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VeicProd {
    /// Tipo de operação realizada (p. ex., venda concessionária, faturamento direto).
    #[serde(rename = "tpOp", default)]
    pub tp_op: Option<String>,

    /// Chassi identificador do veículo.
    #[serde(rename = "chassi", default)]
    pub chassi: Option<String>,

    /// Código identificador correspondente à Cor.
    #[serde(rename = "cCor", default)]
    pub c_cor: Option<String>,

    /// Descrição literal correspondente à Cor.
    #[serde(rename = "xCor", default)]
    pub x_cor: Option<String>,

    /// Potência declarada do motor em cavalos-vapor (CV).
    #[serde(rename = "pot", default)]
    pub pot: Option<String>,

    /// Cilindrada volumétrica declarada do motor.
    #[serde(rename = "cilin", default)]
    pub cilin: Option<String>,

    /// Peso líquido declarado do veículo.
    #[serde(rename = "pesoL", default)]
    pub peso_l: Option<String>,

    /// Peso bruto declarado do veículo.
    #[serde(rename = "pesoB", default)]
    pub peso_b: Option<String>,

    /// Número identificador correspondente à série do motor.
    #[serde(rename = "nSerie", default)]
    pub n_serie: Option<String>,

    /// Tipo de combustível principal utilizado pelo veículo.
    #[serde(rename = "tpComb", default)]
    pub tp_comb: Option<String>,

    /// Número sequencial identificador correspondente ao motor.
    #[serde(rename = "nMotor", default)]
    pub n_motor: Option<String>,

    /// Capacidade Máxima de Tração (CMT) do veículo.
    #[serde(rename = "CMT", default)]
    pub cmt: Option<String>,

    /// Distância entre os eixos declarada.
    #[serde(rename = "dist", default)]
    pub dist: Option<String>,

    /// Ano correspondente ao Modelo de fabricação.
    #[serde(rename = "anoMod", default)]
    pub ano_mod: Option<String>,

    /// Ano correspondente à Fabricação do veículo.
    #[serde(rename = "anoFab", default)]
    pub ano_fab: Option<String>,

    /// Tipo de pintura adotada.
    #[serde(rename = "tpPint", default)]
    pub tp_pint: Option<String>,

    /// Tipo de veículo (automóvel, caminhonete, caminhão, etc.).
    #[serde(rename = "tpVeic", default)]
    pub tp_veic: Option<String>,

    /// Espécie de veículo cadastrada.
    #[serde(rename = "espVeic", default)]
    pub esp_veic: Option<String>,

    /// Código de identificação internacional VIN.
    #[serde(rename = "VIN", default)]
    pub vin: Option<String>,

    /// Condição física estrutural do veículo (p. ex., acabado, inacabado).
    #[serde(rename = "condVeic", default)]
    pub cond_veic: Option<String>,

    /// Código correspondente ao modelo do veículo.
    #[serde(rename = "cMod", default)]
    pub c_mod: Option<String>,

    /// Código correspondente à cor de acordo com a tabela DENATRAN.
    #[serde(rename = "cCorDENATRAN", default)]
    pub c_cor_denatran: Option<String>,

    /// Lotação máxima declarada de passageiros.
    #[serde(rename = "lota", default)]
    pub lota: Option<String>,

    /// Tipo de restrição de tráfego aplicada se aplicável.
    #[serde(rename = "tpRest", default)]
    pub tp_rest: Option<String>,

    /// Texto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// Dados correspondentes a Medicamentos e Insumos Farmacêuticos (`<med>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Med {
    /// Código identificador sanitário emitido pela ANVISA.
    #[serde(rename = "cProdANVISA", default)]
    pub c_prod_anvisa: Option<String>,

    /// Justificativa legal correspondente à isenção de registro se aplicável.
    #[serde(rename = "xMotivoIsencao", default)]
    pub x_motivo_isencao: Option<String>,

    /// Preço Máximo ao Consumidor (PMC) tabelado.
    #[serde(rename = "vPMC", default)]
    pub v_pmc: Option<String>,

    /// Texto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// Dados específicos correspondentes a Armamentos controlados (`<arma>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Arma {
    /// Tipo de arma (p. ex., de uso permitido ou restrito).
    #[serde(rename = "tpArma", default)]
    pub tp_arma: Option<String>,

    /// Número identificador correspondente à série da arma.
    #[serde(rename = "nSerie", default)]
    pub n_serie: Option<String>,

    /// Número identificador correspondente à série do cano.
    #[serde(rename = "nCano", default)]
    pub n_cano: Option<String>,

    /// Descrição detalhada correspondente ao armamento.
    #[serde(rename = "descr", default)]
    pub descr: Option<String>,

    /// Texto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// Informações para faturamento de Combustíveis e Lubrificantes (`<comb>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Comb {
    /// Código correspondente ao produto de acordo com a tabela ANP.
    #[serde(rename = "cProdANP", default)]
    pub c_prod_anp: Option<String>,

    /// Descrição oficial de acordo com a tabela ANP.
    #[serde(rename = "descANP", default)]
    pub desc_anp: Option<String>,

    /// Grupo indicador da origem do combustível (origComb).
    #[serde(rename = "origComb", default)]
    pub orig_comb: Option<Vec<OrigComb>>,

    /// Percentual de mistura do Biodiesel no combustível (pBio).
    #[serde(rename = "pBio", default)]
    pub p_bio: Option<String>,

    /// Percentual de GLP derivado do petróleo se aplicável.
    #[serde(rename = "pGLP", default)]
    pub p_glp: Option<String>,

    /// Percentual de Gás Natural Nacional se aplicável.
    #[serde(rename = "pGNn", default)]
    pub p_gnn: Option<String>,

    /// Percentual de Gás Natural Importado se aplicável.
    #[serde(rename = "pGNi", default)]
    pub p_gni: Option<String>,

    /// Valor de partida correspondente.
    #[serde(rename = "vPart", default)]
    pub v_part: Option<String>,

    /// Código de autorização CODIF se aplicável.
    #[serde(rename = "CODIF", default)]
    pub codif: Option<String>,

    /// Temperatura faturada do combustível.
    #[serde(rename = "qTemp", default)]
    pub q_temp: Option<String>,

    /// Unidade Federativa correspondente ao consumo planejado.
    #[serde(rename = "UFCons", default)]
    pub ufcons: Option<String>,

    /// Texto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    /// Dados correspondentes à CIDE se aplicável.
    #[serde(rename = "CIDE", default)]
    pub cide: Option<Cide>,

    /// Dados correspondentes a encerrantes de bombas se aplicável.
    #[serde(rename = "encerrante", default)]
    pub encerrante: Option<Encerrante>,
}

/// Grupo indicador da origem do combustível (`<origComb>`).
///
/// Mapeia o percentual de mistura e as origens de produção ou importação do combustível.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OrigComb {
    /// Indicador de importação: 0 - Nacional; 1 - Importado.
    #[serde(rename = "indImport", default)]
    pub ind_import: Option<String>,

    /// Código da UF de origem do produtor ou importador (tabela IBGE).
    #[serde(rename = "cUFOrig", default)]
    pub c_uf_orig: Option<String>,

    /// Percentual de mistura de origem (pOrig).
    #[serde(rename = "pOrig", default)]
    pub p_orig: Option<String>,

    /// Conteúdo de texto bruto se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// Contribuição de Intervenção no Domínio Econômico (`<CIDE>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Cide {
    /// Alíquota base para cálculo da CIDE.
    #[serde(rename = "qBCProd", default)]
    pub q_bcprod: Option<String>,

    /// Valor monetário calculado de alíquota CIDE.
    #[serde(rename = "vAliqProd", default)]
    pub v_aliq_prod: Option<String>,

    /// Valor apurado correspondente à CIDE.
    #[serde(rename = "vCIDE", default)]
    pub v_cide: Option<String>,

    /// Texto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// Informações de medições físicas correspondentes a bicos de bomba (`<encerrante>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Encerrante {
    /// Número identificador correspondente ao Bico de abastecimento.
    #[serde(rename = "nBico", default)]
    pub n_bico: Option<String>,

    /// Número identificador correspondente à Bomba de abastecimento.
    #[serde(rename = "nBomba", default)]
    pub n_bomba: Option<String>,

    /// Número identificador correspondente ao Tanque de armazenamento.
    #[serde(rename = "nTanque", default)]
    pub n_tanque: Option<String>,

    /// Valor inicial correspondente ao encerrante.
    #[serde(rename = "vEncIni", default)]
    pub v_enc_ini: Option<String>,

    /// Valor final correspondente ao encerrante.
    #[serde(rename = "vEncFin", default)]
    pub v_enc_fin: Option<String>,

    /// Texto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// Bloco correspondente à devolução de tributos incidentes (`<impostoDevol>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ImpostoDevol {
    /// Alíquota correspondente à devolução de tributos.
    #[serde(rename = "pDevol", default)]
    pub p_devol: Option<String>,

    /// Texto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    /// Dados específicos correspondentes ao IPI devolvido.
    #[serde(rename = "IPI", default)]
    pub ipi: Option<ImpostoDevolNfeIpi>,
}

/// Bloco correspondente aos valores apurados do IPI Devolvido (`<IPI>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ImpostoDevolNfeIpi {
    /// Valor monetário correspondente ao IPI Devolvido.
    #[serde(rename = "vIPIDevol", default)]
    pub v_ipidevol: Option<String>,

    /// Texto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// Bloco de Informações do Transporte da Nota Fiscal (`<transp>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Transp {
    /// Modalidade de contratação de frete utilizada (CIF/FOB, etc.).
    #[serde(rename = "modFrete", default)]
    pub mod_frete: Option<String>,

    /// Número de identificação do vagão se aplicável.
    #[serde(rename = "vagao", default)]
    pub vagao: Option<String>,

    /// Número de identificação da balsa se aplicável.
    #[serde(rename = "balsa", default)]
    pub balsa: Option<String>,

    /// Texto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    /// Dados correspondentes à identificação da transportadora.
    #[serde(rename = "transporta", default)]
    pub transporta: Option<Transporta>,

    /// Dados sobre a retenção de ICMS do serviço de transporte.
    #[serde(rename = "retTransp", default)]
    pub ret_transp: Option<RetTransp>,

    /// Dados sobre o veículo tracionador principal.
    #[serde(rename = "veicTransp", default)]
    pub veic_transp: Option<VeicTransp>,

    /// Lista dos veículos de reboque vinculados.
    #[serde(rename = "reboque", default)]
    pub reboque: Option<Vec<Reboque>>,

    /// Lista de dados de volumes transportados.
    #[serde(rename = "vol", default)]
    pub vol: Option<Vec<Vol>>,
}

/// Dados cadastrais de identificação da Transportadora (`<transporta>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Transporta {
    /// CNPJ da transportadora contratada.
    #[serde(rename = "CNPJ", default)]
    pub cnpj: Option<String>,

    /// CPF da transportadora contratada.
    #[serde(rename = "CPF", default)]
    pub cpf: Option<String>,

    /// Razão social ou nome da transportadora.
    #[serde(rename = "xNome", default)]
    pub x_nome: Option<String>,

    /// Inscrição Estadual relacionada.
    #[serde(rename = "IE", default)]
    pub ie: Option<String>,

    /// Endereço cadastrado correspondente à transportadora.
    #[serde(rename = "xEnder", default)]
    pub x_ender: Option<String>,

    /// Nome do Município correspondente.
    #[serde(rename = "xMun", default)]
    pub x_mun: Option<String>,

    /// Unidade Federativa correspondente.
    #[serde(rename = "UF", default)]
    pub uf: Option<String>,

    /// Texto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// Informações relativas a Retenção de ICMS de Transporte (`<retTransp>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RetTransp {
    /// Valor monetário total correspondente ao serviço faturado.
    #[serde(rename = "vServ", default)]
    pub v_serv: Option<String>,

    /// Base de cálculo monetária correspondente ao serviço de transporte.
    #[serde(rename = "vBCRet", default)]
    pub v_bcret: Option<String>,

    /// Alíquota correspondente à retenção do ICMS.
    #[serde(rename = "pICMSRet", default)]
    pub p_icmsret: Option<String>,

    /// Valor de ICMS retido.
    #[serde(rename = "vICMSRet", default)]
    pub v_icmsret: Option<String>,

    /// Código CFOP aplicável ao serviço.
    #[serde(rename = "CFOP", default)]
    pub cfop: Option<String>,

    /// Código de Município gerador do ICMS de transporte.
    #[serde(rename = "cMunFG", default)]
    pub c_mun_fg: Option<String>,

    /// Texto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// Informações correspondentes ao Veículo de Transporte rodoviário principal (`<veicTransp>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VeicTransp {
    /// Placa do veículo rodoviário tracionador.
    #[serde(rename = "placa", default)]
    pub placa: Option<String>,

    /// Unidade Federativa de emplacamento correspondente.
    #[serde(rename = "UF", default)]
    pub uf: Option<String>,

    /// Registro Nacional de Transportador de Cargas (RNTC).
    #[serde(rename = "RNTC", default)]
    pub rntc: Option<String>,

    /// Texto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// Informações correspondentes a Reboques vinculados ao transporte (`<reboque>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Reboque {
    /// Placa identificadora correspondente ao reboque tracionado.
    #[serde(rename = "placa", default)]
    pub placa: Option<String>,

    /// Unidade Federativa correspondente ao emplacamento.
    #[serde(rename = "UF", default)]
    pub uf: Option<String>,

    /// Registro Nacional correspondente (RNTC).
    #[serde(rename = "RNTC", default)]
    pub rntc: Option<String>,

    /// Texto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// Dados correspondentes a volumes físicos transportados (`<vol>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Vol {
    /// Quantidade de volumes físicos contidos no lote.
    #[serde(rename = "qVol", default)]
    pub q_vol: Option<String>,

    /// Espécie de volumes embalados declarada.
    #[serde(rename = "esp", default)]
    pub esp: Option<String>,

    /// Marca estampada nos volumes.
    #[serde(rename = "marca", default)]
    pub marca: Option<String>,

    /// Número identificador impresso nos volumes.
    #[serde(rename = "nVol", default)]
    pub n_vol: Option<String>,

    /// Peso líquido consolidado declarado.
    #[serde(rename = "pesoL", default)]
    pub peso_l: Option<String>,

    /// Peso bruto consolidado declarado.
    #[serde(rename = "pesoB", default)]
    pub peso_b: Option<String>,

    /// Texto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    /// Lista dos lacres aplicados correspondentes.
    #[serde(rename = "lacres", default)]
    pub lacres: Option<Vec<Lacres>>,
}

/// Dados de Lacres de segurança utilizados nos volumes (`<lacres>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Lacres {
    /// Número identificador correspondente ao Lacre.
    #[serde(rename = "nLacre", default)]
    pub n_lacre: Option<String>,

    /// Texto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// Dados cadastrais de intermediador de transações financeiras ou de entrega (`<infIntermed>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InfIntermed {
    /// CNPJ correspondente ao intermediador.
    #[serde(rename = "CNPJ", default)]
    pub cnpj: Option<String>,

    /// Código identificador cadastrado do intermediador de transporte se aplicável.
    #[serde(rename = "idCadIntTran", default)]
    pub id_cad_int_tran: Option<String>,

    /// Texto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// Bloco de Informações Adicionais vinculadas à nota (`<infAdic>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InfAdic {
    /// Informações adicionais registradas de interesse do fisco.
    #[serde(rename = "infAdFisco", default)]
    pub inf_ad_fisco: Option<String>,

    /// Informações adicionais registradas de interesse de uso interno do contribuinte.
    #[serde(rename = "infCpl", default)]
    pub inf_cpl: Option<String>,

    /// Texto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    /// Coleção de observações do contribuinte registradas.
    #[serde(rename = "obsCont", default)]
    pub obs_cont: Option<Vec<ObsContribuinte>>,

    /// Coleção de observações do fisco registradas.
    #[serde(rename = "obsFisco", default)]
    pub obs_fisco: Option<Vec<ObsFisco>>,

    /// Coleção de referências a processos vinculados.
    #[serde(rename = "procRef", default)]
    pub proc_ref: Option<Vec<ProcRef>>,
}

/// Dados estruturados de observações declaradas pelo Contribuinte (`<obsCont>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ObsContribuinte {
    /// Texto correspondente à observação inserida.
    #[serde(rename = "xTexto", default)]
    pub x_texto: Option<String>,

    /// Nome do campo chave correspondente.
    #[serde(rename = "@xCampo", default)]
    pub x_campo: Option<String>,

    /// Texto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// Dados estruturados de observações declaradas pelo Fisco (`<obsFisco>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ObsFisco {
    /// Texto correspondente à observação fiscal.
    #[serde(rename = "xTexto", default)]
    pub x_texto: Option<String>,

    /// Nome do campo chave correspondente.
    #[serde(rename = "@xCampo", default)]
    pub x_campo: Option<String>,

    /// Texto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// Informações correspondentes a Processos Referenciados administrativamente ou judicialmente (`<procRef>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProcRef {
    /// Número identificador correspondente ao processo.
    #[serde(rename = "nProc", default)]
    pub n_proc: Option<String>,

    /// Origem do processo referenciado (p. ex., SEFAZ, Justiça Federal).
    #[serde(rename = "indProc", default)]
    pub ind_proc: Option<String>,

    /// Texto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// Dados complementares sobre Exportações do Documento Fiscal (`<exporta>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Exporta {
    /// Unidade Federativa onde ocorrerá o embarque internacional.
    #[serde(rename = "UFSaidaPais", default)]
    pub ufsaida_pais: Option<String>,

    /// Local correspondente ao efetivo despacho aduaneiro.
    #[serde(rename = "xLocExporta", default)]
    pub x_loc_exporta: Option<String>,

    /// Local correspondente onde ocorrerá a fiscalização de saída física do País.
    #[serde(rename = "xLocDespacho", default)]
    pub x_loc_despacho: Option<String>,

    /// Texto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// Dados referentes a aquisições institucionais de insumos ou compras (`<compra>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Compra {
    /// Identificador de nota de empenho pública relacionada.
    #[serde(rename = "xNEmp", default)]
    pub x_nemp: Option<String>,

    /// Identificador do Contrato de Compra relacionado.
    #[serde(rename = "xPed", default)]
    pub x_ped: Option<String>,

    /// Identificador correspondente ao Contrato Geral.
    #[serde(rename = "xCont", default)]
    pub x_cont: Option<String>,

    /// Texto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// Bloco de Informações para controle de entrega de Cana-de-Açúcar (`<cana>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Cana {
    /// Safra agrícola vinculada.
    #[serde(rename = "safra", default)]
    pub safra: Option<String>,

    /// Ano e mês correspondente.
    #[serde(rename = "ref", default)]
    pub refer: Option<String>,

    /// Quantidade de cana fornecida no mês.
    #[serde(rename = "qTotMes", default)]
    pub q_tot_mes: Option<String>,

    /// Quantidade de cana fornecida no período anterior.
    #[serde(rename = "qTotAnt", default)]
    pub q_tot_ant: Option<String>,

    /// Quantidade geral apurada acumulada.
    #[serde(rename = "qTotGer", default)]
    pub q_tot_ger: Option<String>,

    /// Valor monetário calculado de fornecimento.
    #[serde(rename = "vFor", default)]
    pub v_for: Option<String>,

    /// Valor de deduções apuradas no fornecimento.
    #[serde(rename = "vTotDed", default)]
    pub v_tot_ded: Option<String>,

    /// Valor líquido final pago ao fornecedor rural.
    #[serde(rename = "vLiqFor", default)]
    pub v_liq_for: Option<String>,

    /// Texto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    /// Diário de fornecimento de cana se aplicável.
    #[serde(rename = "forDia", default)]
    pub for_dia: Option<ForDia>,

    /// Detalhes relativos a deduções apuradas.
    #[serde(rename = "deduc", default)]
    pub deduc: Option<Deduc>,
}

/// Diário detalhado de pesagem de cana-de-açúcar (`<forDia>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ForDia {
    /// Quantidade pesada no dia informado.
    #[serde(rename = "qtde", default)]
    pub qtde: Option<String>,

    /// Dia do fornecimento correspondente.
    #[serde(rename = "@dia", default)]
    pub dia: Option<String>,

    /// Texto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// Dados relativos a Deduções registradas no fornecimento (`<deduc>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Deduc {
    /// Descrição literal correspondente à dedução.
    #[serde(rename = "xDed", default)]
    pub x_ded: Option<String>,

    /// Valor monetário calculado correspondente à dedução realizada.
    #[serde(rename = "vDed", default)]
    pub v_ded: Option<String>,

    /// Texto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// Protocolo de Distribuição e Status de autorização da NF-e (`<protNFe>`).
#[derive(Debug, Serialize, Deserialize)]
pub struct ProtNfe {
    /// Versão declarada no leiaute do protocolo.
    #[serde(rename = "@versao", default)]
    pub versao: Option<String>,
    /// Dados consolidados correspondentes ao protocolo de recepção da NF-e.
    #[serde(rename = "infProt")]
    pub inf_prot: InfProtocolo,
}

/// Informações complementares de distribuição destinadas ao Consumidor Final (`<infNFeSupl>`).
#[derive(Debug, Serialize, Deserialize)]
pub struct InfNfeSupl {
    /// Endereço URL completo contendo o QR Code para consulta direta em portais SEFAZ.
    #[serde(rename = "qrCode", default)]
    pub qr_code: Option<String>,
    /// Endereço URL de consulta manual utilizando a chave de acesso do documento.
    #[serde(rename = "urlChave", default)]
    pub url_chave: Option<String>,
    /// Texto contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// Dados de Nota Fiscal Avulsa emitida por órgãos fazendários (`<avulsa>`).
#[derive(Debug, Serialize, Deserialize)]
pub struct Avulsa {
    /// CNPJ da repartição pública emissora.
    #[serde(rename = "CNPJ", default)]
    pub cnpj: Option<String>,
    /// Nome da repartição.
    #[serde(rename = "xOrgao", default)]
    pub x_orgao: Option<String>,
    /// Matrícula do funcionário.
    #[serde(rename = "matr", default)]
    pub matr: Option<String>,
    /// Nome do funcionário.
    #[serde(rename = "xAgente", default)]
    pub x_agente: Option<String>,
    /// Telefone para contato.
    #[serde(rename = "fone", default)]
    pub fone: Option<String>,
    /// Unidade federativa do órgão emissor.
    #[serde(rename = "UF", default)]
    pub uf: Option<String>,
    /// Número do documento de arrecadação de receita.
    #[serde(rename = "nDAR", default)]
    pub n_dar: Option<String>,
    /// Data de emissão da guia de arrecadação.
    #[serde(rename = "dEmi", default)]
    pub d_emi: Option<String>,
    /// Valor monetário pago na guia de arrecadação.
    #[serde(rename = "vDAR", default)]
    pub v_dar: Option<String>,
    /// Representante legal emissor.
    #[serde(rename = "repEmi", default)]
    pub rep_emi: Option<String>,
    /// Data do efetivo pagamento.
    #[serde(rename = "dPag", default)]
    pub d_pag: Option<String>,
    /// Conteúdo textual do nó.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// Dados do endereço físico do local de retirada da carga (`<retirada>`).
#[derive(Debug, Serialize, Deserialize)]
pub struct Retirada {
    /// CNPJ associado ao ponto de retirada.
    #[serde(rename = "CNPJ", default)]
    pub cnpj: Option<String>,
    /// CPF associado ao ponto de retirada.
    #[serde(rename = "CPF", default)]
    pub cpf: Option<String>,
    /// Razão social ou nome de identificação do ponto de retirada.
    #[serde(rename = "xNome", default)]
    pub x_nome: Option<String>,
    /// Logradouro do endereço.
    #[serde(rename = "xLgr", default)]
    pub x_lgr: Option<String>,
    /// Número do endereço.
    #[serde(rename = "nro", default)]
    pub nro: Option<String>,
    /// Complemento do endereço.
    #[serde(rename = "xCpl", default)]
    pub x_cpl: Option<String>,
    /// Bairro do endereço.
    #[serde(rename = "xBairro", default)]
    pub x_bairro: Option<String>,
    /// Código do Município de acordo com a tabela do IBGE.
    #[serde(rename = "cMun", default)]
    pub c_mun: Option<String>,
    /// Nome do Município correspondente.
    #[serde(rename = "xMun", default)]
    pub x_mun: Option<String>,
    /// Sigla correspondente à Unidade Federativa.
    #[serde(rename = "UF", default)]
    pub uf: Option<String>,
    /// Código de Endereçamento Postal (CEP).
    #[serde(rename = "CEP", default)]
    pub cep: Option<String>,
    /// Código do País de origem.
    #[serde(rename = "cPais", default)]
    pub c_pais: Option<String>,
    /// Nome descritivo do País correspondente.
    #[serde(rename = "xPais", default)]
    pub x_pais: Option<String>,
    /// Telefone cadastrado.
    #[serde(rename = "fone", default)]
    pub fone: Option<String>,
    /// Correio eletrônico cadastrado.
    #[serde(rename = "email", default)]
    pub email: Option<String>,
    /// Inscrição Estadual relacionada.
    #[serde(rename = "IE", default)]
    pub ie: Option<String>,
    /// Conteúdo textual contido no nó se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

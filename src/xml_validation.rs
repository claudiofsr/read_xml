//! # Validador de Integridade de Esquemas XML
//!
//! Código autogerado pelo script `scripts/gerar_validacao.py`. Não edite este arquivo manualmente.

use indicatif::{ProgressBar, ProgressStyle};
use quick_xml::events::Event;
use rayon::prelude::*;
use serde_aux::prelude::serde_introspect;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use crate::XmlPathExt;
use crate::xml_structs::{
    agente, aut_xml, cancelamento, cancelamento_cte, cancelamento_nfe, cobranca, cte,
    cte_detalhamento, cte_evento, efinanceira, endereco, entrega, impostos, integrated_dev_env,
    nfe, nfe_detalhamento, nfe_evento, pagamento, ret_evento,
};

/// Coleta de forma estática todas as tags XML esperadas pelo domínio de NF-e.
fn get_nfe_expected_tags() -> HashSet<&'static str> {
    let mut expected = HashSet::new();
    expected.extend(serde_introspect::<nfe::InfNfe>());
    expected.extend(serde_introspect::<nfe::InfoNfe>());
    expected.extend(serde_introspect::<nfe::Item>());
    expected.extend(serde_introspect::<nfe::Nfe>());
    expected.extend(serde_introspect::<nfe::NfeProc>());
    expected.extend(serde_introspect::<nfe_detalhamento::Adi>());
    expected.extend(serde_introspect::<nfe_detalhamento::Arma>());
    expected.extend(serde_introspect::<nfe_detalhamento::Avulsa>());
    expected.extend(serde_introspect::<nfe_detalhamento::Cana>());
    expected.extend(serde_introspect::<nfe_detalhamento::Cide>());
    expected.extend(serde_introspect::<nfe_detalhamento::Comb>());
    expected.extend(serde_introspect::<nfe_detalhamento::Compra>());
    expected.extend(serde_introspect::<nfe_detalhamento::DFeReferenciado>());
    expected.extend(serde_introspect::<nfe_detalhamento::Deduc>());
    expected.extend(serde_introspect::<nfe_detalhamento::DetExport>());
    expected.extend(serde_introspect::<nfe_detalhamento::Detalhes>());
    expected.extend(serde_introspect::<nfe_detalhamento::Di>());
    expected.extend(serde_introspect::<nfe_detalhamento::Encerrante>());
    expected.extend(serde_introspect::<nfe_detalhamento::ExportInd>());
    expected.extend(serde_introspect::<nfe_detalhamento::Exporta>());
    expected.extend(serde_introspect::<nfe_detalhamento::ForDia>());
    expected.extend(serde_introspect::<nfe_detalhamento::GCred>());
    expected.extend(serde_introspect::<nfe_detalhamento::ImpostoDevol>());
    expected.extend(serde_introspect::<nfe_detalhamento::ImpostoDevolNfeIpi>());
    expected.extend(serde_introspect::<nfe_detalhamento::InfAdic>());
    expected.extend(serde_introspect::<nfe_detalhamento::InfIntermed>());
    expected.extend(serde_introspect::<nfe_detalhamento::InfNfeSupl>());
    expected.extend(serde_introspect::<nfe_detalhamento::Lacres>());
    expected.extend(serde_introspect::<nfe_detalhamento::Med>());
    expected.extend(serde_introspect::<nfe_detalhamento::ObsContribuinte>());
    expected.extend(serde_introspect::<nfe_detalhamento::ObsFisco>());
    expected.extend(serde_introspect::<nfe_detalhamento::ObsItem>());
    expected.extend(serde_introspect::<nfe_detalhamento::OrigComb>());
    expected.extend(serde_introspect::<nfe_detalhamento::ProcRef>());
    expected.extend(serde_introspect::<nfe_detalhamento::Produto>());
    expected.extend(serde_introspect::<nfe_detalhamento::ProtNfe>());
    expected.extend(serde_introspect::<nfe_detalhamento::Rastro>());
    expected.extend(serde_introspect::<nfe_detalhamento::Reboque>());
    expected.extend(serde_introspect::<nfe_detalhamento::RetTransp>());
    expected.extend(serde_introspect::<nfe_detalhamento::Retirada>());
    expected.extend(serde_introspect::<nfe_detalhamento::Transp>());
    expected.extend(serde_introspect::<nfe_detalhamento::Transporta>());
    expected.extend(serde_introspect::<nfe_detalhamento::VeicProd>());
    expected.extend(serde_introspect::<nfe_detalhamento::VeicTransp>());
    expected.extend(serde_introspect::<nfe_detalhamento::Vol>());
    expected.extend(serde_introspect::<nfe_evento::Cte>());
    expected.extend(serde_introspect::<nfe_evento::DetEvento>());
    expected.extend(serde_introspect::<nfe_evento::Evento>());
    expected.extend(serde_introspect::<nfe_evento::InfEvento>());
    expected.extend(serde_introspect::<nfe_evento::InfoNfeEvento>());
    expected.extend(serde_introspect::<nfe_evento::ItemPedido>());
    expected.extend(serde_introspect::<nfe_evento::Mdfe>());
    expected.extend(serde_introspect::<nfe_evento::ModalRodov>());
    expected.extend(serde_introspect::<nfe_evento::ProcEventoNfe>());
    expected.extend(serde_introspect::<cancelamento_nfe::InfoNfeCancel>());
    expected.extend(serde_introspect::<cancelamento_nfe::ProcCancNfe>());
    expected.extend(serde_introspect::<pagamento::Card>());
    expected.extend(serde_introspect::<pagamento::DetalhesPagamento>());
    expected.extend(serde_introspect::<pagamento::Pagamento>());
    adicionar_tags_comuns(&mut expected);
    expected
}

/// Coleta de forma estática todas as tags XML esperadas pelo domínio de CT-e.
fn get_cte_expected_tags() -> HashSet<&'static str> {
    let mut expected = HashSet::new();
    expected.extend(serde_introspect::<cte::Comp>());
    expected.extend(serde_introspect::<cte::Cte>());
    expected.extend(serde_introspect::<cte::CteProc>());
    expected.extend(serde_introspect::<cte::CteSimp>());
    expected.extend(serde_introspect::<cte::CteSimpInfCte>());
    expected.extend(serde_introspect::<cte::CteSimpProc>());
    expected.extend(serde_introspect::<cte::DetCteSimp>());
    expected.extend(serde_introspect::<cte::DetGtv>());
    expected.extend(serde_introspect::<cte::Gtve>());
    expected.extend(serde_introspect::<cte::GtveInfCte>());
    expected.extend(serde_introspect::<cte::GtveProc>());
    expected.extend(serde_introspect::<cte::InfCte>());
    expected.extend(serde_introspect::<cte::InfCteSupl>());
    expected.extend(serde_introspect::<cte::InfDocAntSimp>());
    expected.extend(serde_introspect::<cte::InfEspecie>());
    expected.extend(serde_introspect::<cte::InfNfeTranspParcial>());
    expected.extend(serde_introspect::<cte::InfVeiculoGtve>());
    expected.extend(serde_introspect::<cte::InfoCte>());
    expected.extend(serde_introspect::<cte::ProtCte>());
    expected.extend(serde_introspect::<cte::TotalCteSimp>());
    expected.extend(serde_introspect::<cte::VPrest>());
    expected.extend(serde_introspect::<cte_detalhamento::Aereo>());
    expected.extend(serde_introspect::<cte_detalhamento::Aquav>());
    expected.extend(serde_introspect::<cte_detalhamento::Compl>());
    expected.extend(serde_introspect::<cte_detalhamento::CteLacUnidCarga>());
    expected.extend(serde_introspect::<cte_detalhamento::DetCont>());
    expected.extend(serde_introspect::<cte_detalhamento::DocAnt>());
    expected.extend(serde_introspect::<cte_detalhamento::EmiDocAnt>());
    expected.extend(serde_introspect::<cte_detalhamento::EmiOcc>());
    expected.extend(serde_introspect::<cte_detalhamento::Fluxo>());
    expected.extend(serde_introspect::<cte_detalhamento::IdDocAnt>());
    expected.extend(serde_introspect::<cte_detalhamento::IdDocAntEle>());
    expected.extend(serde_introspect::<cte_detalhamento::IdDocAntPap>());
    expected.extend(serde_introspect::<cte_detalhamento::InfCarga>());
    expected.extend(serde_introspect::<cte_detalhamento::InfCteAnu>());
    expected.extend(serde_introspect::<cte_detalhamento::InfCteComp>());
    expected.extend(serde_introspect::<cte_detalhamento::InfCteMultimodal>());
    expected.extend(serde_introspect::<cte_detalhamento::InfCteNorm>());
    expected.extend(serde_introspect::<cte_detalhamento::InfCteSub>());
    expected.extend(serde_introspect::<cte_detalhamento::InfDoc>());
    expected.extend(serde_introspect::<cte_detalhamento::InfGlobalizado>());
    expected.extend(serde_introspect::<cte_detalhamento::InfModal>());
    expected.extend(serde_introspect::<cte_detalhamento::InfNf>());
    expected.extend(serde_introspect::<cte_detalhamento::InfNfe>());
    expected.extend(serde_introspect::<cte_detalhamento::InfOutros>());
    expected.extend(serde_introspect::<cte_detalhamento::InfOutrosUnidTransp>());
    expected.extend(serde_introspect::<cte_detalhamento::InfPaa>());
    expected.extend(serde_introspect::<cte_detalhamento::InfQ>());
    expected.extend(serde_introspect::<cte_detalhamento::InfSeg>());
    expected.extend(serde_introspect::<cte_detalhamento::InfServVinc>());
    expected.extend(serde_introspect::<cte_detalhamento::InfSolicNff>());
    expected.extend(serde_introspect::<cte_detalhamento::InfUnidCarga>());
    expected.extend(serde_introspect::<cte_detalhamento::InfUnidTransp>());
    expected.extend(serde_introspect::<cte_detalhamento::InfoDeCarga>());
    expected.extend(serde_introspect::<cte_detalhamento::LacUnidCarga>());
    expected.extend(serde_introspect::<cte_detalhamento::LacreAplicado>());
    expected.extend(serde_introspect::<cte_detalhamento::LacreAquav>());
    expected.extend(serde_introspect::<cte_detalhamento::LacreUnidCarga>());
    expected.extend(serde_introspect::<cte_detalhamento::LacresAplicados>());
    expected.extend(serde_introspect::<cte_detalhamento::LacresDeUnidades>());
    expected.extend(serde_introspect::<cte_detalhamento::LacresFisicos>());
    expected.extend(serde_introspect::<cte_detalhamento::LacresSeguranca>());
    expected.extend(serde_introspect::<cte_detalhamento::Multimodal>());
    expected.extend(serde_introspect::<cte_detalhamento::NatCarga>());
    expected.extend(serde_introspect::<cte_detalhamento::ObsCont>());
    expected.extend(serde_introspect::<cte_detalhamento::ObsFisco>());
    expected.extend(serde_introspect::<cte_detalhamento::Occ>());
    expected.extend(serde_introspect::<cte_detalhamento::OutrosInfUnidCarga>());
    expected.extend(serde_introspect::<cte_detalhamento::OutrosLacUnidCarga>());
    expected.extend(serde_introspect::<cte_detalhamento::Paasignature>());
    expected.extend(serde_introspect::<cte_detalhamento::Pass>());
    expected.extend(serde_introspect::<cte_detalhamento::RefNf>());
    expected.extend(serde_introspect::<cte_detalhamento::Rodo>());
    expected.extend(serde_introspect::<cte_detalhamento::RsakeyValue>());
    expected.extend(serde_introspect::<cte_detalhamento::SegMultimodal>());
    expected.extend(serde_introspect::<cte_detalhamento::TarifaAereo>());
    expected.extend(serde_introspect::<cte_detalhamento::TomaIcms>());
    expected.extend(serde_introspect::<cte_detalhamento::UnidCargaAcoplada>());
    expected.extend(serde_introspect::<cte_detalhamento::UnidadeAssociada>());
    expected.extend(serde_introspect::<cte_detalhamento::UnidadeCarga>());
    expected.extend(serde_introspect::<cte_detalhamento::UnidadeTransp>());
    expected.extend(serde_introspect::<cte_detalhamento::VeicNovos>());
    expected.extend(serde_introspect::<cte_evento::DetEvento>());
    expected.extend(serde_introspect::<cte_evento::EvCancCeCte>());
    expected.extend(serde_introspect::<cte_evento::EvCancCte>());
    expected.extend(serde_introspect::<cte_evento::EvCancCteComplementar>());
    expected.extend(serde_introspect::<cte_evento::EvCancInsucessoEntregaCte>());
    expected.extend(serde_introspect::<cte_evento::EvCancPrestDesacordo>());
    expected.extend(serde_introspect::<cte_evento::EvCceCte>());
    expected.extend(serde_introspect::<cte_evento::EvCeCte>());
    expected.extend(serde_introspect::<cte_evento::EvCteAutorizadoMdfe>());
    expected.extend(serde_introspect::<cte_evento::EvCteCanceladoMdfe>());
    expected.extend(serde_introspect::<cte_evento::EvCteComplementar>());
    expected.extend(serde_introspect::<cte_evento::EvCteMultimodal>());
    expected.extend(serde_introspect::<cte_evento::EvCteRedespacho>());
    expected.extend(serde_introspect::<cte_evento::EvCteRedespachoInter>());
    expected.extend(serde_introspect::<cte_evento::EvCteRegPassagem>());
    expected.extend(serde_introspect::<cte_evento::EvCteRegPassagemAuto>());
    expected.extend(serde_introspect::<cte_evento::EvCteSubcontratacao>());
    expected.extend(serde_introspect::<cte_evento::EvCteSubstituido>());
    expected.extend(serde_introspect::<cte_evento::EvEpecCte>());
    expected.extend(serde_introspect::<cte_evento::EvGtveAutorizadoCteOs>());
    expected.extend(serde_introspect::<cte_evento::EvIeCte>());
    expected.extend(serde_introspect::<cte_evento::EvInsucessoEntregaCte>());
    expected.extend(serde_introspect::<cte_evento::EvPrestDesacordo>());
    expected.extend(serde_introspect::<cte_evento::Evento>());
    expected.extend(serde_introspect::<cte_evento::InfCorrecao>());
    expected.extend(serde_introspect::<cte_evento::InfEntregaCeCte>());
    expected.extend(serde_introspect::<cte_evento::InfEvento>());
    expected.extend(serde_introspect::<cte_evento::InfPass>());
    expected.extend(serde_introspect::<cte_evento::InfoCteEvento>());
    expected.extend(serde_introspect::<cte_evento::Mdfe>());
    expected.extend(serde_introspect::<cte_evento::ProcEventoCte>());
    expected.extend(serde_introspect::<cancelamento_cte::InfoCteCancel>());
    expected.extend(serde_introspect::<cancelamento_cte::ProcCancCte>());
    adicionar_tags_comuns(&mut expected);
    expected
}

/// Coleta de forma estática todas as tags XML esperadas pelo domínio de e-Financeira.
fn get_efin_expected_tags() -> HashSet<&'static str> {
    let mut expected = HashSet::new();
    expected.extend(serde_introspect::<efinanceira::BalancoConta>());
    expected.extend(serde_introspect::<efinanceira::Cambio>());
    expected.extend(serde_introspect::<efinanceira::CambioMedJudic>());
    expected.extend(serde_introspect::<efinanceira::Conta>());
    expected.extend(serde_introspect::<efinanceira::ContaMedJudic>());
    expected.extend(serde_introspect::<efinanceira::EFinanceira>());
    expected.extend(serde_introspect::<efinanceira::EnderecoPaisResid>());
    expected.extend(serde_introspect::<efinanceira::EnderecoV2>());
    expected.extend(serde_introspect::<efinanceira::EnderecoV3>());
    expected.extend(serde_introspect::<efinanceira::EstruturaNome>());
    expected.extend(serde_introspect::<efinanceira::EvtMovOpFin>());
    expected.extend(serde_introspect::<efinanceira::Fundo>());
    expected.extend(serde_introspect::<efinanceira::IdeDeclarado>());
    expected.extend(serde_introspect::<efinanceira::IdeDeclaradoNif>());
    expected.extend(serde_introspect::<efinanceira::IdeDeclarante>());
    expected.extend(serde_introspect::<efinanceira::IdeEvento>());
    expected.extend(serde_introspect::<efinanceira::InfoConta>());
    expected.extend(serde_introspect::<efinanceira::InfoContaReportavel>());
    expected.extend(serde_introspect::<efinanceira::InfoEFinanceira>());
    expected.extend(serde_introspect::<efinanceira::Intermediario>());
    expected.extend(serde_introspect::<efinanceira::LocalNascimento>());
    expected.extend(serde_introspect::<efinanceira::MesCaixa>());
    expected.extend(serde_introspect::<efinanceira::MovOpFin>());
    expected.extend(serde_introspect::<efinanceira::NifNomeOutros>());
    expected.extend(serde_introspect::<efinanceira::NifNomeOutrosNomePf>());
    expected.extend(serde_introspect::<efinanceira::NomeDoMeio>());
    expected.extend(serde_introspect::<efinanceira::NomePj>());
    expected.extend(serde_introspect::<efinanceira::NomePjInfoNascimento>());
    expected.extend(serde_introspect::<efinanceira::NomePjInfoNascimentoPaisNasc>());
    expected.extend(serde_introspect::<efinanceira::PaisDeNascimento>());
    expected.extend(serde_introspect::<efinanceira::PaisEnderecoEnderecoOutros>());
    expected.extend(serde_introspect::<efinanceira::PaisNascPaisEndereco>());
    expected.extend(serde_introspect::<efinanceira::PaisNascReportavel>());
    expected.extend(serde_introspect::<efinanceira::PaisResidPaisNacionalidade>());
    expected.extend(serde_introspect::<efinanceira::PgtosAcum>());
    expected.extend(serde_introspect::<efinanceira::PrefixoDoNome>());
    expected.extend(serde_introspect::<efinanceira::PrimeiroNome>());
    expected.extend(serde_introspect::<efinanceira::Proprietarios>());
    expected.extend(serde_introspect::<efinanceira::ProprietariosNif>());
    expected.extend(serde_introspect::<efinanceira::UltimoNomePaisEndereco>());
    expected
}

/// Adiciona as tags comuns de estruturas transversais compartilhadas.
fn adicionar_tags_comuns(expected: &mut HashSet<&'static str>) {
    expected.extend(serde_introspect::<agente::Agente>());
    expected.extend(serde_introspect::<aut_xml::AutXML>());
    expected.extend(serde_introspect::<aut_xml::InfProtocolo>());
    expected.extend(serde_introspect::<aut_xml::InfRespTec>());
    expected.extend(serde_introspect::<endereco::Endereco>());
    expected.extend(serde_introspect::<entrega::ComData>());
    expected.extend(serde_introspect::<entrega::ComHora>());
    expected.extend(serde_introspect::<entrega::Entrega>());
    expected.extend(serde_introspect::<entrega::NoInter>());
    expected.extend(serde_introspect::<entrega::NoPeriodo>());
    expected.extend(serde_introspect::<entrega::SemData>());
    expected.extend(serde_introspect::<entrega::SemHora>());
    expected.extend(serde_introspect::<impostos::Cbs>());
    expected.extend(serde_introspect::<impostos::Cofins>());
    expected.extend(serde_introspect::<impostos::Cofinsaliq>());
    expected.extend(serde_introspect::<impostos::Cofinsnt>());
    expected.extend(serde_introspect::<impostos::Cofinsoutr>());
    expected.extend(serde_introspect::<impostos::Cofinsqtde>());
    expected.extend(serde_introspect::<impostos::Cofinsst>());
    expected.extend(serde_introspect::<impostos::GAjusteCompet>());
    expected.extend(serde_introspect::<impostos::GCbsItem>());
    expected.extend(serde_introspect::<impostos::GCbsTot>());
    expected.extend(serde_introspect::<impostos::GCredPresDet>());
    expected.extend(serde_introspect::<impostos::GCredPresIbsZfm>());
    expected.extend(serde_introspect::<impostos::GCredPresOper>());
    expected.extend(serde_introspect::<impostos::GDevTrib>());
    expected.extend(serde_introspect::<impostos::GDif>());
    expected.extend(serde_introspect::<impostos::GEstornoCred>());
    expected.extend(serde_introspect::<impostos::GIbsCbs>());
    expected.extend(serde_introspect::<impostos::GIbsCbsMono>());
    expected.extend(serde_introspect::<impostos::GIbsCbsMonoTot>());
    expected.extend(serde_introspect::<impostos::GIbsMunItem>());
    expected.extend(serde_introspect::<impostos::GIbsMunTot>());
    expected.extend(serde_introspect::<impostos::GIbsTot>());
    expected.extend(serde_introspect::<impostos::GIbsUfItem>());
    expected.extend(serde_introspect::<impostos::GIbsUfTot>());
    expected.extend(serde_introspect::<impostos::GMonoDif>());
    expected.extend(serde_introspect::<impostos::GMonoPadrao>());
    expected.extend(serde_introspect::<impostos::GMonoRet>());
    expected.extend(serde_introspect::<impostos::GMonoReten>());
    expected.extend(serde_introspect::<impostos::GRed>());
    expected.extend(serde_introspect::<impostos::GTransfCred>());
    expected.extend(serde_introspect::<impostos::GTribCompraGov>());
    expected.extend(serde_introspect::<impostos::GTribRegular>());
    expected.extend(serde_introspect::<impostos::GTribSn>());
    expected.extend(serde_introspect::<impostos::Ibs>());
    expected.extend(serde_introspect::<impostos::Ibscbs>());
    expected.extend(serde_introspect::<impostos::Ibscbstot>());
    expected.extend(serde_introspect::<impostos::Icms>());
    expected.extend(serde_introspect::<impostos::Icms00>());
    expected.extend(serde_introspect::<impostos::Icms02>());
    expected.extend(serde_introspect::<impostos::Icms10>());
    expected.extend(serde_introspect::<impostos::Icms15>());
    expected.extend(serde_introspect::<impostos::Icms20>());
    expected.extend(serde_introspect::<impostos::Icms30>());
    expected.extend(serde_introspect::<impostos::Icms40>());
    expected.extend(serde_introspect::<impostos::Icms45>());
    expected.extend(serde_introspect::<impostos::Icms51>());
    expected.extend(serde_introspect::<impostos::Icms53>());
    expected.extend(serde_introspect::<impostos::Icms60>());
    expected.extend(serde_introspect::<impostos::Icms61>());
    expected.extend(serde_introspect::<impostos::Icms70>());
    expected.extend(serde_introspect::<impostos::Icms90>());
    expected.extend(serde_introspect::<impostos::IcmsOutraUf>());
    expected.extend(serde_introspect::<impostos::IcmsTot>());
    expected.extend(serde_introspect::<impostos::Icmspart>());
    expected.extend(serde_introspect::<impostos::Icmssn>());
    expected.extend(serde_introspect::<impostos::Icmssn101>());
    expected.extend(serde_introspect::<impostos::Icmssn102>());
    expected.extend(serde_introspect::<impostos::Icmssn201>());
    expected.extend(serde_introspect::<impostos::Icmssn202>());
    expected.extend(serde_introspect::<impostos::Icmssn500>());
    expected.extend(serde_introspect::<impostos::Icmssn900>());
    expected.extend(serde_introspect::<impostos::Icmsst>());
    expected.extend(serde_introspect::<impostos::Icmsufdest>());
    expected.extend(serde_introspect::<impostos::Icmsuffim>());
    expected.extend(serde_introspect::<impostos::Ii>());
    expected.extend(serde_introspect::<impostos::Imposto>());
    expected.extend(serde_introspect::<impostos::ImpostoSeletivo>());
    expected.extend(serde_introspect::<impostos::Ipi>());
    expected.extend(serde_introspect::<impostos::Ipint>());
    expected.extend(serde_introspect::<impostos::Ipitrib>());
    expected.extend(serde_introspect::<impostos::IsTot>());
    expected.extend(serde_introspect::<impostos::Issqn>());
    expected.extend(serde_introspect::<impostos::Issqntot>());
    expected.extend(serde_introspect::<impostos::Pis>());
    expected.extend(serde_introspect::<impostos::Pisaliq>());
    expected.extend(serde_introspect::<impostos::Pisnt>());
    expected.extend(serde_introspect::<impostos::Pisoutr>());
    expected.extend(serde_introspect::<impostos::Pisqtde>());
    expected.extend(serde_introspect::<impostos::Pisst>());
    expected.extend(serde_introspect::<impostos::RetTrib>());
    expected.extend(serde_introspect::<impostos::Total>());
    expected.extend(serde_introspect::<integrated_dev_env::GCompraGov>());
    expected.extend(serde_introspect::<integrated_dev_env::GPagAntecipado>());
    expected.extend(serde_introspect::<integrated_dev_env::Ide>());
    expected.extend(serde_introspect::<integrated_dev_env::NFref>());
    expected.extend(serde_introspect::<integrated_dev_env::RefEcf>());
    expected.extend(serde_introspect::<integrated_dev_env::RefNf>());
    expected.extend(serde_introspect::<integrated_dev_env::RefNfp>());
    expected.extend(serde_introspect::<ret_evento::RetEvento>());
    expected.extend(serde_introspect::<ret_evento::RetInfEvento>());
    // Adiciona cancelamento e cobrança comuns
    expected.extend(serde_introspect::<cancelamento::Cancelamento>());
    expected.extend(serde_introspect::<cancelamento::Retencao>());
    expected.extend(serde_introspect::<cancelamento::InfoCancelamento>());
    expected.extend(serde_introspect::<cobranca::Cobranca>());
    expected.extend(serde_introspect::<cobranca::Dup>());
    expected.extend(serde_introspect::<cobranca::Fat>());
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum XmlDomain {
    Nfe,
    Cte,
    EFinanceira,
    Desconhecido,
}

#[derive(Debug)]
enum XmlCheckOutcome {
    Analisado {
        domain: XmlDomain,
        root_tag: String,
        total_tags: usize,
        unmapped: BTreeSet<String>,
    },
    ErroLeitura,
}

static NFE_EXPECTED_TAGS: LazyLock<HashSet<&'static str>> = LazyLock::new(get_nfe_expected_tags);
static CTE_EXPECTED_TAGS: LazyLock<HashSet<&'static str>> = LazyLock::new(get_cte_expected_tags);
static EFIN_EXPECTED_TAGS: LazyLock<HashSet<&'static str>> = LazyLock::new(get_efin_expected_tags);

#[inline]
fn is_root_or_xmldsig_tag(tag: &str) -> bool {
    matches!(
        tag,
        "nfeProc"
            | "NFe"
            | "cteProc"
            | "CTe"
            | "eFinanceira"
            | "procEventoCTe"
            | "eventoCTe"
            | "procEventoNFe"
            | "evento"
            | "procCancCTe"
            | "procCancNFe"
            | "GTVeProc"
            | "GTVe"
            | "cteSimpProc"
            | "CTeSimp"
    ) || tag.starts_with("Signature")
        || tag.starts_with("SignedInfo")
        || tag.starts_with("CanonicalizationMethod")
        || tag.starts_with("SignatureMethod")
        || tag.starts_with("Reference")
        || tag.starts_with("Transforms")
        || tag.starts_with("Transform")
        || tag.starts_with("DigestMethod")
        || tag.starts_with("DigestValue")
        || tag.starts_with("SignatureValue")
        || tag.starts_with("KeyInfo")
        || tag.starts_with("X509Data")
        || tag.starts_with("X509Certificate")
}

/// Realiza varredura SAX em passada única (detectando a tag raiz no próprio stream),
/// contabilizando total de tags lidas e tags não mapeadas.
fn inspecionar_xml(xml_path: &Path) -> XmlCheckOutcome {
    let Ok(mut reader) = xml_path.open_xml_reader_with_capacity(64 * 1024) else {
        return XmlCheckOutcome::ErroLeitura;
    };

    let mut buf = Vec::with_capacity(512);
    let mut domain: Option<XmlDomain> = None;
    let mut root_tag_name = String::new();
    let mut expected_tags: Option<&HashSet<&'static str>> = None;
    let mut total_tags = 0usize;
    let mut unmapped = BTreeSet::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) | Ok(Event::Empty(e)) => {
                let raw_name = e.name().into_inner();
                let clean_tag = raw_name.split(':').next_back().unwrap_or(raw_name);
                total_tags += 1;

                if domain.is_none() {
                    root_tag_name = clean_tag.to_string();
                    let (dom, exp) = match clean_tag {
                        "nfeProc" | "NFe" | "procEventoNFe" | "evento" | "procCancNFe" => {
                            (XmlDomain::Nfe, Some(&*NFE_EXPECTED_TAGS))
                        }
                        "cteProc" | "CTe" | "GTVeProc" | "GTVe" | "cteSimpProc" | "CTeSimp"
                        | "procEventoCTe" | "eventoCTe" | "procCancCTe" => {
                            (XmlDomain::Cte, Some(&*CTE_EXPECTED_TAGS))
                        }
                        "eFinanceira" => (XmlDomain::EFinanceira, Some(&*EFIN_EXPECTED_TAGS)),
                        _ => (XmlDomain::Desconhecido, None),
                    };
                    domain = Some(dom);
                    expected_tags = exp;
                    if exp.is_none() {
                        break;
                    }
                } else if let Some(expected) = expected_tags
                    && !expected.contains(clean_tag)
                    && !is_root_or_xmldsig_tag(clean_tag)
                {
                    unmapped.insert(clean_tag.to_string());
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => return XmlCheckOutcome::ErroLeitura,
            _ => {}
        }
        buf.clear();
    }

    match domain {
        Some(dom) => XmlCheckOutcome::Analisado {
            domain: dom,
            root_tag: root_tag_name,
            total_tags,
            unmapped,
        },
        None => XmlCheckOutcome::ErroLeitura,
    }
}

/// Executa a varredura SAX concorrente em lote sobre todos os XMLs, exibe as
/// inconsistências agrupadas de forma determinística e imprime um resumo completo.
pub fn verificar_inconsistencias_de_xml(xml_paths: &[PathBuf]) {
    let total = xml_paths.len();
    let num_char = total.to_string().chars().count();

    let template_bar =
        "{prefix:<10.bold.dim} {spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos:>nchar}/{len:>nchar} ({eta})"
        .replace("nchar", &num_char.to_string());

    let style = match ProgressStyle::default_bar().template(&template_bar) {
        Ok(s) => s.progress_chars("#>-"),
        Err(_) => ProgressStyle::default_bar(),
    };

    let pb = ProgressBar::new(total as u64);
    pb.set_style(style);
    pb.set_prefix("check xml");

    let resultados: Vec<(&PathBuf, XmlCheckOutcome)> = xml_paths
        .par_iter()
        .map(|path| {
            pb.inc(1);
            (path, inspecionar_xml(path))
        })
        .collect();

    pb.finish_and_clear();

    let mut total_tags_lidas = 0usize;
    let mut nfe_count = 0usize;
    let mut cte_count = 0usize;
    let mut efin_count = 0usize;
    let mut desconhecidos_count = 0usize;
    let mut erros_count = 0usize;
    let mut conformes_count = 0usize;
    let mut inconsistentes_count = 0usize;

    let mut por_root_tag: BTreeMap<String, usize> = BTreeMap::new();
    let mut unknown_roots: BTreeMap<String, usize> = BTreeMap::new();
    let mut erros_amostra: Vec<&PathBuf> = Vec::new();
    let mut tags_unicas_nao_mapeadas: BTreeSet<String> = BTreeSet::new();
    let mut agrupado: HashMap<BTreeSet<String>, Vec<PathBuf>> = HashMap::new();

    for (path, outcome) in resultados {
        match outcome {
            XmlCheckOutcome::ErroLeitura => {
                erros_count += 1;
                if erros_amostra.len() < 5 {
                    erros_amostra.push(path);
                }
            }
            XmlCheckOutcome::Analisado {
                domain,
                root_tag,
                total_tags,
                unmapped,
            } => {
                total_tags_lidas += total_tags;
                match domain {
                    XmlDomain::Nfe => nfe_count += 1,
                    XmlDomain::Cte => cte_count += 1,
                    XmlDomain::EFinanceira => efin_count += 1,
                    XmlDomain::Desconhecido => {
                        desconhecidos_count += 1;
                        *unknown_roots.entry(root_tag).or_default() += 1;
                        continue;
                    }
                }

                *por_root_tag.entry(root_tag).or_default() += 1;

                if unmapped.is_empty() {
                    conformes_count += 1;
                } else {
                    inconsistentes_count += 1;
                    tags_unicas_nao_mapeadas.extend(unmapped.iter().cloned());
                    agrupado.entry(unmapped).or_default().push(path.clone());
                }
            }
        }
    }

    let mut agrupado_vec: Vec<(BTreeSet<String>, Vec<PathBuf>)> = agrupado.into_iter().collect();
    agrupado_vec.sort_by(|a, b| b.1.len().cmp(&a.1.len()).then_with(|| a.0.cmp(&b.0)));
    let num_grupos = agrupado_vec.len();

    for (tags, mut sorted_paths) in agrupado_vec {
        sorted_paths.sort();

        println!(
            "Aviso: Tags verificadas em arquivos XML não mapeadas em Structs: {:?}",
            tags
        );
        println!(
            "Total de {} arquivo(s) com esta inconsistência:",
            sorted_paths.len()
        );
        for path in sorted_paths {
            println!("  - {:?}", path);
        }
        println!();
    }

    let elegiveis = nfe_count + cte_count + efin_count;
    let pct = |n: usize| -> f64 {
        if elegiveis == 0 {
            0.0
        } else {
            (n as f64 * 100.0) / elegiveis as f64
        }
    };

    println!("=== Resumo da Verificação de Consistência (-k) ===");
    println!("Arquivos XML encontrados   : {total}");
    println!("Arquivos XML elegíveis     : {elegiveis}");
    println!("Total de tags inspecionadas: {total_tags_lidas}");
    println!(
        "Conformes (sem pendências) : {conformes_count} ({:.2}%)",
        pct(conformes_count)
    );
    println!(
        "Com tags não mapeadas      : {inconsistentes_count} ({:.2}%)",
        pct(inconsistentes_count)
    );
    println!("Ignorados (raiz desconhec.): {desconhecidos_count}");
    println!("Erros de leitura / XML     : {erros_count}");

    println!("\nDistribuição por domínio:");
    println!("  NF-e        : {nfe_count}");
    println!("  CT-e        : {cte_count}");
    println!("  e-Financeira: {efin_count}");

    if !por_root_tag.is_empty() {
        let mut roots_sorted: Vec<(&String, &usize)> = por_root_tag.iter().collect();
        roots_sorted.sort_by(|a, b| b.1.cmp(a.1).then_with(|| a.0.cmp(b.0)));

        println!("\nDetalhamento por tag raiz:");
        for (tag, qtd) in roots_sorted {
            println!("  {tag:<14}: {qtd}");
        }
    }

    if !unknown_roots.is_empty() {
        println!("\nTags raiz ignoradas:");
        for (tag, qtd) in &unknown_roots {
            println!("  {tag:<14}: {qtd}");
        }
    }

    if !erros_amostra.is_empty() {
        println!("\nAmostra de arquivos com erro de leitura/XML:");
        for p in erros_amostra {
            println!("  - {:?}", p);
        }
    }

    if tags_unicas_nao_mapeadas.is_empty() {
        println!(
            "\nStatus: 100% dos XMLs elegíveis estão em conformidade com as structs mapeadas."
        );
    } else {
        println!("\nGrupos de inconsistência   : {num_grupos}");
        println!(
            "Tags únicas não mapeadas ({}): {:?}",
            tags_unicas_nao_mapeadas.len(),
            tags_unicas_nao_mapeadas
        );
    }
    println!();
}

/// Varre o XML físico e retorna o conjunto de tags não mapeadas nas structs (ou `None` se conforme).
pub fn obter_tags_nao_mapeadas(xml_path: &Path) -> Option<HashSet<String>> {
    match inspecionar_xml(xml_path) {
        XmlCheckOutcome::Analisado { unmapped, .. } if !unmapped.is_empty() => {
            Some(unmapped.into_iter().collect())
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn create_temp_xml(filename: &str, content: &str) -> std::path::PathBuf {
        let mut path = std::env::temp_dir();
        path.push(filename);
        fs::write(&path, content).expect("Falha ao criar arquivo XML temporário de teste");
        path
    }

    #[test]
    fn test_xml_validation_complete() {
        let xml_valido_content = r#"<?xml version="1.0" encoding="utf-8"?>
        <nfeProc versao="4.00" xmlns="http://www.portalfiscal.inf.br/nfe">
            <NFe>
                <infNFe Id="NFe3522" versao="4.00">
                    <ide><cUF>35</cUF></ide>
                    <emit><CNPJ>00000000000100</CNPJ></emit>
                    <total><ICMSTot><vProd>150.00</vProd></ICMSTot></total>
                </infNFe>
            </NFe>
        </nfeProc>"#;

        let xml_incompleto_content = r#"<?xml version="1.0" encoding="utf-8"?>
        <nfeProc versao="4.00" xmlns="http://www.portalfiscal.inf.br/nfe">
            <NFe>
                <infNFe Id="NFe3522" versao="4.00">
                    <ide><cUF>35</cUF></ide>
                    <tagInexistenteDeTeste>valor_teste</tagInexistenteDeTeste>
                    <outroElementoFicticio>outro_valor</outroElementoFicticio>
                </infNFe>
            </NFe>
        </nfeProc>"#;

        let xml_desconhecido_content = r#"<?xml version="1.0" encoding="utf-8"?>
        <OutroDocumento><campo>valor</campo></OutroDocumento>"#;

        let path_valido = create_temp_xml("nfe_valido_teste.xml", xml_valido_content);
        let path_incompleto = create_temp_xml("nfe_incompleto_teste.xml", xml_incompleto_content);
        let path_desconhecido = create_temp_xml("desconhecido_teste.xml", xml_desconhecido_content);

        assert!(obter_tags_nao_mapeadas(&path_valido).is_none());
        assert!(obter_tags_nao_mapeadas(&path_desconhecido).is_none());

        let res_incompleto = obter_tags_nao_mapeadas(&path_incompleto);
        assert!(res_incompleto.is_some());

        let unmapped_tags = res_incompleto.unwrap();
        assert_eq!(unmapped_tags.len(), 2);
        assert!(unmapped_tags.contains("tagInexistenteDeTeste"));
        assert!(unmapped_tags.contains("outroElementoFicticio"));

        let _ = fs::remove_file(path_valido);
        let _ = fs::remove_file(path_incompleto);
        let _ = fs::remove_file(path_desconhecido);
    }
}

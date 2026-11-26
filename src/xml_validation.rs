//! # Validador de Integridade de Esquemas XML
//!
//! Código autogerado pelo script `scripts/gerar_validacao.py`. Não edite este arquivo manualmente.

use indicatif::{ProgressBar, ProgressStyle};
use quick_xml::events::Event;
use serde_aux::prelude::serde_introspect;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::path::PathBuf;
use std::sync::LazyLock;

use crate::xml_structs::{
    agente, aut_xml, cancelamento, cancelamento_cte, cancelamento_nfe, cobranca, cte,
    cte_detalhamento, cte_evento, efinanceira, endereco, entrega, impostos, integrated_dev_env,
    nfe, nfe_detalhamento, nfe_evento, pagamento, ret_evento,
};
use crate::{XmlBufReadExt, for_each_xml, partition_sources_with_xml};

/// Coleta de forma estática todas as tags XML esperadas pelo domínio de NF-e.
fn get_nfe_expected_tags() -> HashSet<&'static str> {
    let mut expected = HashSet::new();
    expected.extend(serde_introspect::<nfe::Agropecuario>());
    expected.extend(serde_introspect::<nfe::Defensivo>());
    expected.extend(serde_introspect::<nfe::GPgtoVinc>());
    expected.extend(serde_introspect::<nfe::GuiaTransito>());
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
    expected.extend(serde_introspect::<nfe_evento::Ctg>());
    expected.extend(serde_introspect::<nfe_evento::DetEvento>());
    expected.extend(serde_introspect::<nfe_evento::Evento>());
    expected.extend(serde_introspect::<nfe_evento::InfEvento>());
    expected.extend(serde_introspect::<nfe_evento::InfoNfeEvento>());
    expected.extend(serde_introspect::<nfe_evento::ItemPedido>());
    expected.extend(serde_introspect::<nfe_evento::Mdfe>());
    expected.extend(serde_introspect::<nfe_evento::ModalOutro>());
    expected.extend(serde_introspect::<nfe_evento::ModalRodov>());
    expected.extend(serde_introspect::<nfe_evento::ProcEventoNfe>());
    expected.extend(serde_introspect::<nfe_evento::RespPedido>());
    expected.extend(serde_introspect::<cancelamento_nfe::InfoNfeCancel>());
    expected.extend(serde_introspect::<cancelamento_nfe::ProcCancNfe>());
    expected.extend(serde_introspect::<pagamento::Card>());
    expected.extend(serde_introspect::<pagamento::DetalhesPagamento>());
    expected.extend(serde_introspect::<pagamento::Pagamento>());
    expected.extend(["gPgto"]);
    adicionar_tags_comuns(&mut expected);
    expected
}

/// Coleta de forma estática todas as tags XML esperadas pelo domínio de CT-e.
fn get_cte_expected_tags() -> HashSet<&'static str> {
    let mut expected = HashSet::new();
    expected.extend(serde_introspect::<cte::Comp>());
    expected.extend(serde_introspect::<cte::Cte>());
    expected.extend(serde_introspect::<cte::CteOs>());
    expected.extend(serde_introspect::<cte::CteOsInfCte>());
    expected.extend(serde_introspect::<cte::CteOsProc>());
    expected.extend(serde_introspect::<cte::CteProc>());
    expected.extend(serde_introspect::<cte::CteSimp>());
    expected.extend(serde_introspect::<cte::CteSimpInfCte>());
    expected.extend(serde_introspect::<cte::CteSimpProc>());
    expected.extend(serde_introspect::<cte::DetCteSimp>());
    expected.extend(serde_introspect::<cte::DetGtv>());
    expected.extend(serde_introspect::<cte::GMultaJuros>());
    expected.extend(serde_introspect::<cte::Gtve>());
    expected.extend(serde_introspect::<cte::GtveInfCte>());
    expected.extend(serde_introspect::<cte::GtveProc>());
    expected.extend(serde_introspect::<cte::InfCte>());
    expected.extend(serde_introspect::<cte::InfCteOsNorm>());
    expected.extend(serde_introspect::<cte::InfCteSupl>());
    expected.extend(serde_introspect::<cte::InfDocAntSimp>());
    expected.extend(serde_introspect::<cte::InfDocRefOs>());
    expected.extend(serde_introspect::<cte::InfEspecie>());
    expected.extend(serde_introspect::<cte::InfGtveOs>());
    expected.extend(serde_introspect::<cte::InfNfeTranspParcial>());
    expected.extend(serde_introspect::<cte::InfServicoOs>());
    expected.extend(serde_introspect::<cte::InfVeiculoGtve>());
    expected.extend(serde_introspect::<cte::InfoCte>());
    expected.extend(serde_introspect::<cte::PgtoVinc>());
    expected.extend(serde_introspect::<cte::PgtoVincItem>());
    expected.extend(serde_introspect::<cte::ProtCte>());
    expected.extend(serde_introspect::<cte::SegOs>());
    expected.extend(serde_introspect::<cte::TotalCteSimp>());
    expected.extend(serde_introspect::<cte::VPrest>());
    expected.extend(serde_introspect::<cte_detalhamento::Aereo>());
    expected.extend(serde_introspect::<cte_detalhamento::Aquav>());
    expected.extend(serde_introspect::<cte_detalhamento::Balsa>());
    expected.extend(serde_introspect::<cte_detalhamento::Compl>());
    expected.extend(serde_introspect::<cte_detalhamento::CteLacUnidCarga>());
    expected.extend(serde_introspect::<cte_detalhamento::DetCont>());
    expected.extend(serde_introspect::<cte_detalhamento::DocAnt>());
    expected.extend(serde_introspect::<cte_detalhamento::EmiDocAnt>());
    expected.extend(serde_introspect::<cte_detalhamento::EmiOcc>());
    expected.extend(serde_introspect::<cte_detalhamento::Ferrov>());
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
    expected.extend(serde_introspect::<cte_detalhamento::InfFretamentoOs>());
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
    expected.extend(serde_introspect::<cte_detalhamento::InfTotAp>());
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
    expected.extend(serde_introspect::<cte_detalhamento::Peri>());
    expected.extend(serde_introspect::<cte_detalhamento::RefNf>());
    expected.extend(serde_introspect::<cte_detalhamento::Rodo>());
    expected.extend(serde_introspect::<cte_detalhamento::RodoOs>());
    expected.extend(serde_introspect::<cte_detalhamento::RsakeyValue>());
    expected.extend(serde_introspect::<cte_detalhamento::SegMultimodal>());
    expected.extend(serde_introspect::<cte_detalhamento::TarifaAereo>());
    expected.extend(serde_introspect::<cte_detalhamento::TomaIcms>());
    expected.extend(serde_introspect::<cte_detalhamento::TrafMut>());
    expected.extend(serde_introspect::<cte_detalhamento::UnidCargaAcoplada>());
    expected.extend(serde_introspect::<cte_detalhamento::UnidadeAssociada>());
    expected.extend(serde_introspect::<cte_detalhamento::UnidadeCarga>());
    expected.extend(serde_introspect::<cte_detalhamento::UnidadeTransp>());
    expected.extend(serde_introspect::<cte_detalhamento::VeicNovos>());
    expected.extend(serde_introspect::<cte_detalhamento::VeicRodoOs>());
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
    expected.extend(serde_introspect::<cte_evento::EvGtveCancCteOs>());
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
    expected.extend(["chCTe", "chNFe", "gPgto"]);
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
    expected.extend(serde_introspect::<efinanceira::EnderecoEfin>());
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
    expected.extend(serde_introspect::<efinanceira::LoteEventoItem>());
    expected.extend(serde_introspect::<efinanceira::LoteEventos>());
    expected.extend(serde_introspect::<efinanceira::LoteEventosLista>());
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
    expected.extend(["caixaAnual", "movOpFinAnual"]);
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
    expected.extend(serde_introspect::<impostos::GAlcZfmCbs>());
    expected.extend(serde_introspect::<impostos::GCbsItem>());
    expected.extend(serde_introspect::<impostos::GCbsSn>());
    expected.extend(serde_introspect::<impostos::GCbsTot>());
    expected.extend(serde_introspect::<impostos::GCredPresDet>());
    expected.extend(serde_introspect::<impostos::GCredPresIbsZfm>());
    expected.extend(serde_introspect::<impostos::GCredPresOper>());
    expected.extend(serde_introspect::<impostos::GDevTrib>());
    expected.extend(serde_introspect::<impostos::GDif>());
    expected.extend(serde_introspect::<impostos::GEstornoCred>());
    expected.extend(serde_introspect::<impostos::GIbsCbs>());
    expected.extend(serde_introspect::<impostos::GIbsCbsMono>());
    expected.extend(serde_introspect::<impostos::GIbsCbsMonoAd>());
    expected.extend(serde_introspect::<impostos::GIbsCbsMonoTot>());
    expected.extend(serde_introspect::<impostos::GIbsMunItem>());
    expected.extend(serde_introspect::<impostos::GIbsMunTot>());
    expected.extend(serde_introspect::<impostos::GIbsSn>());
    expected.extend(serde_introspect::<impostos::GIbsTot>());
    expected.extend(serde_introspect::<impostos::GIbsUfItem>());
    expected.extend(serde_introspect::<impostos::GIbsUfTot>());
    expected.extend(serde_introspect::<impostos::GIcmsPrevistoPagtoAntecip>());
    expected.extend(serde_introspect::<impostos::GMonoDif>());
    expected.extend(serde_introspect::<impostos::GMonoPadrao>());
    expected.extend(serde_introspect::<impostos::GMonoRet>());
    expected.extend(serde_introspect::<impostos::GMonoReten>());
    expected.extend(serde_introspect::<impostos::GRed>());
    expected.extend(serde_introspect::<impostos::GTransfCred>());
    expected.extend(serde_introspect::<impostos::GTribCompraGov>());
    expected.extend(serde_introspect::<impostos::GTribRegular>());
    expected.extend(serde_introspect::<impostos::GTribSn>());
    expected.extend(serde_introspect::<impostos::GTribSnTot>());
    expected.extend(serde_introspect::<impostos::GpBioDiferenca>());
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
    expected.extend(serde_introspect::<integrated_dev_env::InfPercurso>());
    expected.extend(serde_introspect::<integrated_dev_env::NFref>());
    expected.extend(serde_introspect::<integrated_dev_env::RefEcf>());
    expected.extend(serde_introspect::<integrated_dev_env::RefNf>());
    expected.extend(serde_introspect::<integrated_dev_env::RefNfp>());
    expected.extend(serde_introspect::<ret_evento::RetEvento>());
    expected.extend(serde_introspect::<ret_evento::RetInfEvento>());
    expected.extend([
        "ISUFemit",
        "enderDest",
        "enderEmit",
        "enderExped",
        "enderReceb",
        "enderReme",
        "enderToma",
        "gICMSPrevistoPagtoAntecip",
        "pDif",
        "toma03",
        "toma04",
        "toma3",
        "toma4",
        "tomaTerceiro",
        "vBCUFDest",
    ]);
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
            | "cteOSProc"
            | "CTeOS"
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

/// Realiza varredura SAX em passada única sobre um reader XML genérico (`BufRead`).
fn inspecionar_xml_reader<R: std::io::BufRead>(
    mut reader: quick_xml::Reader<quick_xml::encoding::DecodingReader<R>>,
) -> Result<XmlCheckOutcome, quick_xml::Error> {
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
                        | "cteOSProc" | "CTeOS" | "procEventoCTe" | "eventoCTe" | "procCancCTe" => {
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
            Err(err) => return Err(err),
            _ => {}
        }
        buf.clear();
    }

    Ok(match domain {
        Some(dom) => XmlCheckOutcome::Analisado {
            domain: dom,
            root_tag: root_tag_name,
            total_tags,
            unmapped,
        },
        None => XmlCheckOutcome::ErroLeitura,
    })
}

/// Realiza varredura SAX em passada única sobre bytes em memória (entradas de `.zip` e `.xml`s soltos).
fn inspecionar_xml_bytes(bytes: &[u8]) -> XmlCheckOutcome {
    let Ok(reader) = bytes.into_xml_reader() else {
        return XmlCheckOutcome::ErroLeitura;
    };
    match inspecionar_xml_reader(reader) {
        Ok(outcome) => outcome,
        Err(quick_xml::Error::Encoding(_)) => {
            let mut decoder = quick_xml::encoding::DecodingReader::new(bytes);
            decoder.set_encoding(encoding_rs::WINDOWS_1252);
            let fallback = quick_xml::Reader::from_reader(decoder);
            inspecionar_xml_reader(fallback).unwrap_or(XmlCheckOutcome::ErroLeitura)
        }
        Err(_) => XmlCheckOutcome::ErroLeitura,
    }
}

/// Executa a varredura SAX concorrente em `.zip`s (sem extrair no disco) e em arquivos `.xml` soltos.
pub fn verificar_inconsistencias(zip_paths: &[PathBuf]) -> crate::XmlParserResult<()> {
    let (active_zips, skipped_zips, total_entries) = partition_sources_with_xml(zip_paths);
    if active_zips.is_empty() {
        eprintln!(
            "Aviso: nenhum arquivo .zip com XML ou .xml solto foi encontrado no caminho informado."
        );
        return Ok(());
    }
    let num_char = total_entries.max(1).to_string().chars().count();

    let template_bar =
        "{prefix:<10.bold.dim} {spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos:>nchar}/{len:>nchar} ({eta})"
            .replace("nchar", &num_char.to_string());

    let style = match ProgressStyle::default_bar().template(&template_bar) {
        Ok(s) => s.progress_chars("#>-"),
        Err(_) => ProgressStyle::default_bar(),
    };

    let pb = ProgressBar::new(total_entries);
    pb.set_style(style);
    pb.set_prefix("check zip");

    thread_local! {
        static LOCAL_STATS: std::cell::RefCell<Option<ValidationStats>> =
            const { std::cell::RefCell::new(None) };
    }

    for_each_xml(&active_zips, Some(&pb), |entry_label, bytes| {
        let outcome = inspecionar_xml_bytes(bytes);
        LOCAL_STATS.with(|cell| {
            let mut opt = cell.borrow_mut();
            let stats = opt.get_or_insert_with(ValidationStats::default);
            stats.record(entry_label, outcome);
        });
    })?;

    pb.finish_and_clear();
    let global_stats = std::sync::Mutex::new(ValidationStats::default());
    rayon::broadcast(|_| {
        LOCAL_STATS.with(|cell| {
            if let Some(local) = cell.borrow_mut().take()
                && let Ok(mut g) = global_stats.lock()
            {
                g.merge(local);
            }
        });
    });
    LOCAL_STATS.with(|cell| {
        if let Some(local) = cell.borrow_mut().take()
            && let Ok(mut g) = global_stats.lock()
        {
            g.merge(local);
        }
    });

    let stats = global_stats
        .into_inner()
        .expect("Lock poisoned in verificar_inconsistencias");
    exibir_resumo_consistencia(active_zips.len(), skipped_zips, stats);
    Ok(())
}

#[derive(Default)]
struct ValidationStats {
    total: usize,
    total_tags_lidas: usize,
    nfe_count: usize,
    cte_count: usize,
    efin_count: usize,
    desconhecidos_count: usize,
    erros_count: usize,
    conformes_count: usize,
    inconsistentes_count: usize,
    por_root_tag: BTreeMap<String, usize>,
    unknown_roots: BTreeMap<String, usize>,
    erros_amostra: Vec<PathBuf>,
    tags_unicas_nao_mapeadas: BTreeSet<String>,
    agrupado: HashMap<BTreeSet<String>, Vec<PathBuf>>,
}

impl ValidationStats {
    fn record(&mut self, entry_label: &str, outcome: XmlCheckOutcome) {
        self.total += 1;
        match outcome {
            XmlCheckOutcome::ErroLeitura => {
                self.erros_count += 1;
                if self.erros_amostra.len() < 5 {
                    self.erros_amostra.push(PathBuf::from(entry_label));
                }
            }
            XmlCheckOutcome::Analisado {
                domain,
                root_tag,
                total_tags,
                unmapped,
            } => {
                self.total_tags_lidas += total_tags;
                match domain {
                    XmlDomain::Nfe => self.nfe_count += 1,
                    XmlDomain::Cte => self.cte_count += 1,
                    XmlDomain::EFinanceira => self.efin_count += 1,
                    XmlDomain::Desconhecido => {
                        self.desconhecidos_count += 1;
                        *self.unknown_roots.entry(root_tag).or_default() += 1;
                        return;
                    }
                }

                *self.por_root_tag.entry(root_tag).or_default() += 1;

                if unmapped.is_empty() {
                    self.conformes_count += 1;
                } else {
                    self.inconsistentes_count += 1;
                    self.tags_unicas_nao_mapeadas
                        .extend(unmapped.iter().cloned());
                    self.agrupado
                        .entry(unmapped)
                        .or_default()
                        .push(PathBuf::from(entry_label));
                }
            }
        }
    }

    fn merge(&mut self, other: Self) {
        self.total += other.total;
        self.total_tags_lidas += other.total_tags_lidas;
        self.nfe_count += other.nfe_count;
        self.cte_count += other.cte_count;
        self.efin_count += other.efin_count;
        self.desconhecidos_count += other.desconhecidos_count;
        self.erros_count += other.erros_count;
        self.conformes_count += other.conformes_count;
        self.inconsistentes_count += other.inconsistentes_count;

        for (k, v) in other.por_root_tag {
            *self.por_root_tag.entry(k).or_default() += v;
        }
        for (k, v) in other.unknown_roots {
            *self.unknown_roots.entry(k).or_default() += v;
        }
        for p in other.erros_amostra {
            if self.erros_amostra.len() < 5 {
                self.erros_amostra.push(p);
            }
        }
        self.tags_unicas_nao_mapeadas
            .extend(other.tags_unicas_nao_mapeadas);
        for (k, v) in other.agrupado {
            self.agrupado.entry(k).or_default().extend(v);
        }
    }
}

fn exibir_resumo_consistencia(zips_com_xml: usize, zips_ignorados: usize, stats: ValidationStats) {
    let ValidationStats {
        total,
        total_tags_lidas,
        nfe_count,
        cte_count,
        efin_count,
        desconhecidos_count,
        erros_count,
        conformes_count,
        inconsistentes_count,
        por_root_tag,
        unknown_roots,
        erros_amostra,
        tags_unicas_nao_mapeadas,
        agrupado,
    } = stats;

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
    println!("Arquivos ZIP com XML       : {zips_com_xml}");
    println!("ZIPs ignorados (sem XML)   : {zips_ignorados}");
    println!("Entradas XML inspecionadas : {total}");
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

/// Varre bytes XML em memória e retorna as tags não mapeadas (ou `None` se conforme).
pub fn obter_tags_nao_mapeadas_bytes(bytes: &[u8]) -> Option<HashSet<String>> {
    match inspecionar_xml_bytes(bytes) {
        XmlCheckOutcome::Analisado { unmapped, .. } if !unmapped.is_empty() => {
            Some(unmapped.into_iter().collect())
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

        assert!(obter_tags_nao_mapeadas_bytes(xml_valido_content.as_bytes()).is_none());
        assert!(obter_tags_nao_mapeadas_bytes(xml_desconhecido_content.as_bytes()).is_none());

        let res_incompleto = obter_tags_nao_mapeadas_bytes(xml_incompleto_content.as_bytes());
        assert!(res_incompleto.is_some());

        let unmapped_tags = res_incompleto.unwrap();
        assert_eq!(unmapped_tags.len(), 2);
        assert!(unmapped_tags.contains("tagInexistenteDeTeste"));
        assert!(unmapped_tags.contains("outroElementoFicticio"));
    }
}

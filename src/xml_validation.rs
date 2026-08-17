//! # Validador de Integridade de Esquemas XML
//!
//! Código autogerado pelo script `gerar_validacao.py`. Não edite este arquivo manualmente.

use indicatif::{ProgressBar, ProgressStyle};
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use rayon::prelude::*;
use serde_aux::prelude::serde_introspect;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::path::{Path, PathBuf};

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
    expected.extend(serde_introspect::<nfe_detalhamento::Deduc>());
    expected.extend(serde_introspect::<nfe_detalhamento::DetExport>());
    expected.extend(serde_introspect::<nfe_detalhamento::Detalhes>());
    expected.extend(serde_introspect::<nfe_detalhamento::Di>());
    expected.extend(serde_introspect::<nfe_detalhamento::Encerrante>());
    expected.extend(serde_introspect::<nfe_detalhamento::ExportInd>());
    expected.extend(serde_introspect::<nfe_detalhamento::Exporta>());
    expected.extend(serde_introspect::<nfe_detalhamento::ForDia>());
    expected.extend(serde_introspect::<nfe_detalhamento::ImpostoDevol>());
    expected.extend(serde_introspect::<nfe_detalhamento::ImpostoDevolNfeIpi>());
    expected.extend(serde_introspect::<nfe_detalhamento::InfAdic>());
    expected.extend(serde_introspect::<nfe_detalhamento::InfIntermed>());
    expected.extend(serde_introspect::<nfe_detalhamento::InfNfeSupl>());
    expected.extend(serde_introspect::<nfe_detalhamento::Lacres>());
    expected.extend(serde_introspect::<nfe_detalhamento::Med>());
    expected.extend(serde_introspect::<nfe_detalhamento::ObsContribuinte>());
    expected.extend(serde_introspect::<nfe_detalhamento::ObsFisco>());
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
    expected.extend(serde_introspect::<nfe_evento::Mdfe>());
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
    expected.extend(serde_introspect::<cte::InfCte>());
    expected.extend(serde_introspect::<cte::InfCteSupl>());
    expected.extend(serde_introspect::<cte::InfoCte>());
    expected.extend(serde_introspect::<cte::ProtCte>());
    expected.extend(serde_introspect::<cte::VPrest>());
    expected.extend(serde_introspect::<cte_detalhamento::Compl>());
    expected.extend(serde_introspect::<cte_detalhamento::CteLacUnidCarga>());
    expected.extend(serde_introspect::<cte_detalhamento::DocAnt>());
    expected.extend(serde_introspect::<cte_detalhamento::EmiDocAnt>());
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
    expected.extend(serde_introspect::<cte_detalhamento::InfServVinc>());
    expected.extend(serde_introspect::<cte_detalhamento::InfSolicNff>());
    expected.extend(serde_introspect::<cte_detalhamento::InfUnidCarga>());
    expected.extend(serde_introspect::<cte_detalhamento::InfUnidTransp>());
    expected.extend(serde_introspect::<cte_detalhamento::InfoDeCarga>());
    expected.extend(serde_introspect::<cte_detalhamento::LacUnidCarga>());
    expected.extend(serde_introspect::<cte_detalhamento::LacreAplicado>());
    expected.extend(serde_introspect::<cte_detalhamento::LacreUnidCarga>());
    expected.extend(serde_introspect::<cte_detalhamento::LacresAplicados>());
    expected.extend(serde_introspect::<cte_detalhamento::LacresDeUnidades>());
    expected.extend(serde_introspect::<cte_detalhamento::LacresFisicos>());
    expected.extend(serde_introspect::<cte_detalhamento::LacresSeguranca>());
    expected.extend(serde_introspect::<cte_detalhamento::ObsCont>());
    expected.extend(serde_introspect::<cte_detalhamento::ObsFisco>());
    expected.extend(serde_introspect::<cte_detalhamento::OutrosInfUnidCarga>());
    expected.extend(serde_introspect::<cte_detalhamento::OutrosLacUnidCarga>());
    expected.extend(serde_introspect::<cte_detalhamento::Paasignature>());
    expected.extend(serde_introspect::<cte_detalhamento::Pass>());
    expected.extend(serde_introspect::<cte_detalhamento::RefNf>());
    expected.extend(serde_introspect::<cte_detalhamento::Rodo>());
    expected.extend(serde_introspect::<cte_detalhamento::RsakeyValue>());
    expected.extend(serde_introspect::<cte_detalhamento::TomaIcms>());
    expected.extend(serde_introspect::<cte_detalhamento::UnidCargaAcoplada>());
    expected.extend(serde_introspect::<cte_detalhamento::UnidadeAssociada>());
    expected.extend(serde_introspect::<cte_detalhamento::UnidadeCarga>());
    expected.extend(serde_introspect::<cte_detalhamento::UnidadeTransp>());
    expected.extend(serde_introspect::<cte_detalhamento::VeicNovos>());
    expected.extend(serde_introspect::<cte_evento::DetEvento>());
    expected.extend(serde_introspect::<cte_evento::EvCancCte>());
    expected.extend(serde_introspect::<cte_evento::EvCceCte>());
    expected.extend(serde_introspect::<cte_evento::EvCteAutorizadoMdfe>());
    expected.extend(serde_introspect::<cte_evento::EvCteComplementar>());
    expected.extend(serde_introspect::<cte_evento::EvCteMultimodal>());
    expected.extend(serde_introspect::<cte_evento::EvCteRedespacho>());
    expected.extend(serde_introspect::<cte_evento::EvCteRedespachoInter>());
    expected.extend(serde_introspect::<cte_evento::EvCteRegPassagemAuto>());
    expected.extend(serde_introspect::<cte_evento::EvCteSubcontratacao>());
    expected.extend(serde_introspect::<cte_evento::EvCteSubstituido>());
    expected.extend(serde_introspect::<cte_evento::Evento>());
    expected.extend(serde_introspect::<cte_evento::InfCorrecao>());
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

/// Adiciona as tags comuns de estruturas transversais compartilhas.
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
    expected.extend(serde_introspect::<impostos::GCbsTot>());
    expected.extend(serde_introspect::<impostos::GIbsMunTot>());
    expected.extend(serde_introspect::<impostos::GIbsTot>());
    expected.extend(serde_introspect::<impostos::GIbsUfTot>());
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

/// Detecta dinamicamente a tag de abertura raiz do XML de modo resiliente.
fn detect_root_tag(xml_path: &Path) -> Option<String> {
    let mut reader = Reader::from_file(xml_path).ok()?;
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) | Ok(Event::Empty(e)) => {
                let tag_name = String::from_utf8_lossy(e.name().as_ref()).into_owned();
                let clean_tag = tag_name
                    .split(':')
                    .next_back()
                    .unwrap_or(&tag_name)
                    .to_string();
                return Some(clean_tag);
            }
            Ok(Event::Eof) => break,
            _ => {}
        }
        buf.clear();
    }
    None
}

/// Executa a varredura SAX concorrente em lote sobre todos os XMLs e exibe as
/// inconsistências de forma agrupada e determinística, com barra de progresso ativa.
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

    let resultados: Vec<(PathBuf, BTreeSet<String>)> = xml_paths
        .par_iter()
        .filter_map(|path| {
            pb.inc(1);
            let unmapped = obter_tags_nao_mapeadas(path)?;
            let btree_set: BTreeSet<String> = unmapped.into_iter().collect();
            Some((path.clone(), btree_set))
        })
        .collect();

    pb.finish_and_clear(); // Limpa a barra de progresso antes de imprimir o relatório

    if resultados.is_empty() {
        return;
    }

    let mut agrupado: HashMap<BTreeSet<String>, Vec<PathBuf>> = HashMap::new();
    for (path, tags) in resultados {
        agrupado.entry(tags).or_default().push(path);
    }

    // Converte o mapa para um vetor para permitir a ordenação estável
    let mut agrupado_vec: Vec<(BTreeSet<String>, Vec<PathBuf>)> = agrupado.into_iter().collect();

    // Ordena em ordem decrescente de ocorrências (b.1.len().cmp(&a.1.len()))
    // e usa o conjunto de tags (BTreeSet) como critério de desempate estável
    agrupado_vec.sort_by(|a, b| b.1.len().cmp(&a.1.len()).then_with(|| a.0.cmp(&b.0)));

    // 3. Exibe as inconsistências agrupadas em ordem decrescente de ocorrências
    for (tags, mut sorted_paths) in agrupado_vec {
        sorted_paths.sort(); // Ordenação alfabética determinística dos caminhos

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
        println!(); // Linha em branco separadora de grupos
    }
}

/// Varre o XML físico e retorna a lista de tags que não estão mapeadas em nosso conjunto de structs.
pub fn obter_tags_nao_mapeadas(xml_path: &Path) -> Option<HashSet<String>> {
    let root_tag = detect_root_tag(xml_path)?;

    let expected_tags = match root_tag.as_str() {
        "nfeProc" | "NFe" | "procEventoNfe" | "procCancNfe" => get_nfe_expected_tags(),
        "cteProc" | "CTe" | "procEventoCte" | "procCancCte" => get_cte_expected_tags(),
        "eFinanceira" => get_efin_expected_tags(),
        _ => return None,
    };

    let mut reader = Reader::from_file(xml_path).ok()?;
    let mut buf = Vec::new();
    let mut unmapped = HashSet::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) | Ok(Event::Empty(e)) => {
                let tag_name = String::from_utf8_lossy(e.name().as_ref()).into_owned();
                let clean_tag = tag_name.split(':').next_back().unwrap_or(&tag_name);

                if !expected_tags.contains(clean_tag)
                    && clean_tag != "nfeProc"
                    && clean_tag != "cteProc"
                    && clean_tag != "eFinanceira"
                    && !clean_tag.starts_with("Signature")
                    && !clean_tag.starts_with("SignedInfo")
                    && !clean_tag.starts_with("CanonicalizationMethod")
                    && !clean_tag.starts_with("SignatureMethod")
                    && !clean_tag.starts_with("Reference")
                    && !clean_tag.starts_with("Transforms")
                    && !clean_tag.starts_with("Transform")
                    && !clean_tag.starts_with("DigestMethod")
                    && !clean_tag.starts_with("DigestValue")
                    && !clean_tag.starts_with("SignatureValue")
                    && !clean_tag.starts_with("KeyInfo")
                    && !clean_tag.starts_with("X509Data")
                    && !clean_tag.starts_with("X509Certificate")
                {
                    unmapped.insert(clean_tag.to_string());
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => return None,
            _ => {}
        }
        buf.clear();
    }

    if unmapped.is_empty() {
        None
    } else {
        Some(unmapped)
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

        let path_valido = create_temp_xml("nfe_valido_teste.xml", xml_valido_content);
        let path_incompleto = create_temp_xml("nfe_incompleto_teste.xml", xml_incompleto_content);

        let res_valido = obter_tags_nao_mapeadas(&path_valido);
        assert!(res_valido.is_none());

        let res_incompleto = obter_tags_nao_mapeadas(&path_incompleto);
        assert!(res_incompleto.is_some());

        let unmapped_tags = res_incompleto.unwrap();
        assert_eq!(unmapped_tags.len(), 2);
        assert!(unmapped_tags.contains("tagInexistenteDeTeste"));
        assert!(unmapped_tags.contains("outroElementoFicticio"));

        let _ = fs::remove_file(path_valido);
        let _ = fs::remove_file(path_incompleto);
    }
}

#!/usr/bin/env python3
import os
import re

# Caminhos dos diretórios do projeto
XML_STRUCTS_DIR = "src/xml_structs"
OUTPUT_FILE = "src/xml_validation.rs"

# Classificação dos módulos por domínio
DOMINIOS = {
    "comum": ["agente", "aut_xml", "endereco", "entrega", "impostos", "integrated_dev_env", "ret_evento"],
    "nfe": ["nfe", "nfe_detalhamento", "nfe_evento", "cancelamento_nfe", "pagamento"],
    "cte": ["cte", "cte_detalhamento", "cte_evento", "cancelamento_cte"],
    "efin": ["efinanceira"]
}

def obter_structs_do_modulo(modulo):
    filepath = os.path.join(XML_STRUCTS_DIR, f"{modulo}.rs")
    if not os.path.exists(filepath):
        return []
    
    structs = []
    pattern = re.compile(r"^\s*pub\s+struct\s+(\w+)")
    
    with open(filepath, "r", encoding="utf-8") as f:
        for line in f:
            if "#[cfg(test)]" in line:
                break
                
            match = pattern.match(line)
            if match:
                structs.append(match.group(1))
    return sorted(structs)

def gerar_codigo_introspeccao(dominio_chaves):
    linhas = []
    for modulo in dominio_chaves:
        structs = obter_structs_do_modulo(modulo)
        for s in structs:
            linhas.append(f"    expected.extend(serde_introspect::<{modulo}::{s}>());")
    return "\n".join(linhas)

def main():
    # Geração dos blocos de código para cada domínio
    comum_code = gerar_codigo_introspeccao(DOMINIOS["comum"])
    nfe_code = gerar_codigo_introspeccao(DOMINIOS["nfe"])
    cte_code = gerar_codigo_introspeccao(DOMINIOS["cte"])
    efin_code = gerar_codigo_introspeccao(DOMINIOS["efin"])

    # Template do arquivo xml_validation.rs
    template = f"""//! # Validador de Integridade de Esquemas XML
//!
//! Código autogerado pelo script `gerar_validacao.py`. Não edite este arquivo manualmente.

use std::collections::{{HashSet, BTreeSet, HashMap}};
use std::path::{{Path, PathBuf}};
use rayon::prelude::*;
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use serde_aux::prelude::serde_introspect;
use indicatif::{{ProgressBar, ProgressStyle}};

use crate::xml_structs::{{
    agente, aut_xml, cancelamento, cancelamento_cte, cancelamento_nfe, cobranca,
    cte, cte_detalhamento, cte_evento, efinanceira, endereco, entrega, impostos,
    integrated_dev_env, nfe, nfe_detalhamento, nfe_evento, pagamento, ret_evento,
}};

/// Coleta de forma estática todas as tags XML esperadas pelo domínio de NF-e.
fn get_nfe_expected_tags() -> HashSet<&'static str> {{
    let mut expected = HashSet::new();
{nfe_code}
    adicionar_tags_comuns(&mut expected);
    expected
}}

/// Coleta de forma estática todas as tags XML esperadas pelo domínio de CT-e.
fn get_cte_expected_tags() -> HashSet<&'static str> {{
    let mut expected = HashSet::new();
{cte_code}
    adicionar_tags_comuns(&mut expected);
    expected
}}

/// Coleta de forma estática todas as tags XML esperadas pelo domínio de e-Financeira.
fn get_efin_expected_tags() -> HashSet<&'static str> {{
    let mut expected = HashSet::new();
{efin_code}
    expected
}}

/// Adiciona as tags comuns de estruturas transversais compartilhas.
fn adicionar_tags_comuns(expected: &mut HashSet<&'static str>) {{
{comum_code}
    // Adiciona cancelamento e cobrança comuns
    expected.extend(serde_introspect::<cancelamento::Cancelamento>());
    expected.extend(serde_introspect::<cancelamento::Retencao>());
    expected.extend(serde_introspect::<cancelamento::InfoCancelamento>());
    expected.extend(serde_introspect::<cobranca::Cobranca>());
    expected.extend(serde_introspect::<cobranca::Dup>());
    expected.extend(serde_introspect::<cobranca::Fat>());
}}

/// Detecta dinamicamente a tag de abertura raiz do XML de modo resiliente.
fn detect_root_tag(xml_path: &Path) -> Option<String> {{
    let mut reader = Reader::from_file(xml_path).ok()?;
    let mut buf = Vec::new();
    loop {{
        match reader.read_event_into(&mut buf) {{
            Ok(Event::Start(e)) | Ok(Event::Empty(e)) => {{
                let tag_name = String::from_utf8_lossy(e.name().as_ref()).into_owned();
                let clean_tag = tag_name.split(':').next_back().unwrap_or(&tag_name).to_string();
                return Some(clean_tag);
            }}
            Ok(Event::Eof) => break,
            _ => {{}}
        }}
        buf.clear();
    }}
    None
}}

/// Executa a varredura SAX concorrente em lote sobre todos os XMLs e exibe as
/// inconsistências de forma agrupada e determinística, com barra de progresso ativa.
pub fn verificar_inconsistencias_de_xml(xml_paths: &[PathBuf]) {{
    let total = xml_paths.len();
    let num_char = total.to_string().chars().count();
    
    let template_bar =
        "{{prefix:<10.bold.dim}} {{spinner:.green}} [{{elapsed_precise}}] [{{wide_bar:.cyan/blue}}] {{pos:>nchar}}/{{len:>nchar}} ({{eta}})"
        .replace("nchar", &num_char.to_string());

    let style = match ProgressStyle::default_bar().template(&template_bar) {{
        Ok(s) => s.progress_chars("#>-"),
        Err(_) => ProgressStyle::default_bar(),
    }};

    let pb = ProgressBar::new(total as u64);
    pb.set_style(style);
    pb.set_prefix("check xml");

    let resultados: Vec<(PathBuf, BTreeSet<String>)> = xml_paths
        .par_iter()
        .filter_map(|path| {{
            pb.inc(1);
            let unmapped = obter_tags_nao_mapeadas(path)?;
            let btree_set: BTreeSet<String> = unmapped.into_iter().collect();
            Some((path.clone(), btree_set))
        }})
        .collect();

    pb.finish_and_clear(); // Limpa a barra de progresso antes de imprimir o relatório

    if resultados.is_empty() {{
        return;
    }}

    let mut agrupado: HashMap<BTreeSet<String>, Vec<PathBuf>> = HashMap::new();
    for (path, tags) in resultados {{
        agrupado.entry(tags).or_default().push(path);
    }}

    // Converte o mapa para um vetor para permitir a ordenação estável
    let mut agrupado_vec: Vec<(BTreeSet<String>, Vec<PathBuf>)> = agrupado.into_iter().collect();

    // Ordena em ordem decrescente de ocorrências (b.1.len().cmp(&a.1.len()))
    // e usa o conjunto de tags (BTreeSet) como critério de desempate estável
    agrupado_vec.sort_by(|a, b| {{
        b.1.len().cmp(&a.1.len())
            .then_with(|| a.0.cmp(&b.0))
    }});

    // 3. Exibe as inconsistências agrupadas em ordem decrescente de ocorrências
    for (tags, mut sorted_paths) in agrupado_vec {{
        sorted_paths.sort(); // Ordenação alfabética determinística dos caminhos

        println!("Aviso: Tags verificadas em arquivos XML não mapeadas em Structs: {{:?}}", tags);
        println!("Total de {{}} arquivo(s) com esta inconsistência:", sorted_paths.len());        
        for path in sorted_paths {{
            println!("  - {{:?}}", path);
        }}
        println!(); // Linha em branco separadora de grupos
    }}
}}

/// Varre o XML físico e retorna a lista de tags que não estão mapeadas em nosso conjunto de structs.
pub fn obter_tags_nao_mapeadas(xml_path: &Path) -> Option<HashSet<String>> {{
    let root_tag = detect_root_tag(xml_path)?;

    let expected_tags = match root_tag.as_str() {{
        "nfeProc" | "NFe" | "procEventoNfe" | "procCancNfe" => get_nfe_expected_tags(),
        "cteProc" | "CTe" | "procEventoCte" | "procCancCte" => get_cte_expected_tags(),
        "eFinanceira" => get_efin_expected_tags(),
        _ => return None,
    }};

    let mut reader = Reader::from_file(xml_path).ok()?;
    let mut buf = Vec::new();
    let mut unmapped = HashSet::new();

    loop {{
        match reader.read_event_into(&mut buf) {{
            Ok(Event::Start(e)) | Ok(Event::Empty(e)) => {{
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
                {{
                    unmapped.insert(clean_tag.to_string());
                }}
            }}
            Ok(Event::Eof) => break,
            Err(_) => return None,
            _ => {{}}
        }}
        buf.clear();
    }}

    if unmapped.is_empty() {{
        None
    }} else {{
        Some(unmapped)
    }}
}}

#[cfg(test)]
mod tests {{
    use super::*;
    use std::fs;

    fn create_temp_xml(filename: &str, content: &str) -> std::path::PathBuf {{
        let mut path = std::env::temp_dir();
        path.push(filename);
        fs::write(&path, content).expect("Falha ao criar arquivo XML temporário de teste");
        path
    }}

    #[test]
    fn test_xml_validation_complete() {{
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
    }}
}}
"""

    with open(OUTPUT_FILE, "w", encoding="utf-8") as out:
        out.write(template)
    
    print(f"Sucesso: O arquivo '{OUTPUT_FILE}' foi gerado dinamicamente com base em suas structs.")

if __name__ == "__main__":
    main()

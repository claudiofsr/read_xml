//! # Assinaturas Digitais XML (XMLDSig)
//!
//! Este módulo gerencia a desserialização e o mapeamento das assinaturas digitais
//! baseadas no padrão XMLDSig (`<Signature>`), utilizadas para garantir a integridade,
//! a autoria e o não-retrocesso dos documentos fiscais eletrônicos emitidos (NFe, CTe e eventos).

use serde::{Deserialize, Serialize};

/// Assinatura Digital do Documento Fiscal Eletrônico (`<Signature>`).
///
/// Estrutura que representa o nó padrão de assinatura digital XMLDSig
/// utilizado para garantir a validade e a integridade jurídica das transações fiscais.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Signature {
    // =========================================================================
    // 1. ATRIBUTOS DO NÓ XML
    // =========================================================================
    /// Namespace XML correspondente à assinatura.
    #[serde(rename = "@xmlns", default)]
    pub xmlns: Option<String>,

    /// Caminho do esquema de validação XML se presente.
    #[serde(rename = "@xsi:schemaLocation", default)]
    pub xsi_schema_location: Option<String>,

    /// Identificador exclusivo da assinatura.
    #[serde(rename = "@Id", default)]
    pub id: Option<String>,

    /// Namespace XML correspondente à instância Schema.
    #[serde(rename = "@xmlns:xsi", default)]
    pub xmlns_xsi: Option<String>,

    // =========================================================================
    // 2. TEXTO E CAMPOS OPCIONAIS
    // =========================================================================
    /// Conteúdo de texto bruto associado ao nó XML se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    // =========================================================================
    // 3. CAMPOS ESTREITAMENTE OBRIGATÓRIOS
    // =========================================================================
    /// Informações cobertas pela assinatura digital.
    #[serde(rename = "SignedInfo")]
    pub signed_info: SignedInfo,

    /// Valor criptográfico da assinatura digital gerada.
    #[serde(rename = "SignatureValue")]
    pub signature_value: SignatureValue,

    /// Informações sobre a chave pública e os certificados digitais.
    #[serde(rename = "KeyInfo")]
    pub key_info: KeyInfo,
}

/// Bloco de Informações Cobertas pela Assinatura (`<SignedInfo>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SignedInfo {
    // =========================================================================
    // 1. ATRIBUTOS DO NÓ XML
    // =========================================================================
    /// Identificador do bloco de dados assinado.
    #[serde(rename = "@Id", default)]
    pub id: Option<String>,

    // =========================================================================
    // 2. TEXTO E CAMPOS OPCIONAIS
    // =========================================================================
    /// Conteúdo de texto bruto se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    // =========================================================================
    // 3. CAMPOS ESTREITAMENTE OBRIGATÓRIOS
    // =========================================================================
    /// Método de canonização XML utilizado para padronizar o documento antes de assinar.
    #[serde(rename = "CanonicalizationMethod")]
    pub canonicalization_method: CanonicalizationMethod,

    /// Algoritmo criptográfico utilizado para assinar o hash.
    #[serde(rename = "SignatureMethod")]
    pub signature_method: SignatureMethod,

    /// Referência de vinculação ao documento assinado.
    #[serde(rename = "Reference")]
    pub reference: Reference,
}

/// Método de Canonização XML (`<CanonicalizationMethod>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CanonicalizationMethod {
    /// URI correspondente ao algoritmo de canonização adotado.
    #[serde(rename = "@Algorithm", default)]
    pub algorithm: Option<String>,
}

/// Algoritmo de Assinatura Criptográfica (`<SignatureMethod>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SignatureMethod {
    /// URI do algoritmo de assinatura adotado (p. ex., RSA-SHA1).
    #[serde(rename = "@Algorithm", default)]
    pub algorithm: Option<String>,
}

/// Referência de Vinculação de Assinatura (`<Reference>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Reference {
    // =========================================================================
    // 1. ATRIBUTOS DO NÓ XML
    // =========================================================================
    /// Identificador exclusivo da referência.
    #[serde(rename = "@Id", default)]
    pub id: Option<String>,

    /// URI de referência ao nó de dados assinado.
    #[serde(rename = "@URI", default)]
    pub uri: Option<String>,

    /// Tipo de referência se aplicável.
    #[serde(rename = "@Type", default)]
    pub reference_type: Option<String>,

    // =========================================================================
    // 2. TEXTO E CAMPOS OPCIONAIS
    // =========================================================================
    /// Valor do resumo digital (hash) calculado para a referência.
    #[serde(rename = "DigestValue", default)]
    pub digest_value: Option<String>,

    /// Conteúdo de texto bruto se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    // =========================================================================
    // 3. CAMPOS ESTREITAMENTE OBRIGATÓRIOS
    // =========================================================================
    /// Conjunto de transformações XML aplicadas antes da geração do hash.
    #[serde(rename = "Transforms")]
    pub transforms: Transforms,

    /// Algoritmo de resumo digital adotado para gerar o hash.
    #[serde(rename = "DigestMethod")]
    pub digest_method: DigestMethod,
}

/// Bloco de Transformações XML (`<Transforms>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Transforms {
    /// Conteúdo de texto se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    /// Lista de transformações aplicadas.
    #[serde(rename = "Transform", default)]
    pub transform: Vec<Transform>,
}

/// Detalhes de Transformação de Assinatura (`<Transform>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Transform {
    /// Lista de expressões XPath de corte se houver.
    #[serde(rename = "XPath", default)]
    pub xpath: Option<Vec<String>>,

    /// URI correspondente ao algoritmo de transformação adotado.
    #[serde(rename = "@Algorithm", default)]
    pub algorithm: Option<String>,

    /// Conteúdo de texto se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// Algoritmo de Resumo Digital (`<DigestMethod>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DigestMethod {
    /// URI correspondente ao algoritmo de resumo digital adotado (p. ex., SHA1).
    #[serde(rename = "@Algorithm", default)]
    pub algorithm: Option<String>,
}

/// Bloco contendo o Valor da Assinatura Digital (`<SignatureValue>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SignatureValue {
    /// Identificador exclusivo do nó se aplicável.
    #[serde(rename = "@Id", default)]
    pub id: Option<String>,

    /// Conteúdo bruto em Base64 correspondente ao valor criptográfico da assinatura.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// Bloco de Informações de Chave Pública (`<KeyInfo>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KeyInfo {
    // =========================================================================
    // 1. ATRIBUTOS DO NÓ XML
    // =========================================================================
    /// Identificador exclusivo do nó se aplicável.
    #[serde(rename = "@Id", default)]
    pub id: Option<String>,

    // =========================================================================
    // 2. TEXTO E CAMPOS OPCIONAIS
    // =========================================================================
    /// Conteúdo de texto se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    // =========================================================================
    // 3. CAMPOS ESTREITAMENTE OBRIGATÓRIOS
    // =========================================================================
    /// Dados correspondentes ao certificado digital X.509.
    #[serde(rename = "X509Data")]
    pub x509_data: X509Data,
}

/// Bloco de Dados do Certificado Digital X.509 (`<X509Data>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct X509Data {
    /// Certificado digital correspondente em formato Base64.
    #[serde(rename = "X509Certificate", default)]
    pub x509_certificate: Option<String>,

    /// Conteúdo de texto bruto associado se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// Bloco de Assinatura do Protocolo de Autorização (`<protNFe/Signature>` ou `<protCTe/Signature>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProtSignature {
    // =========================================================================
    // 1. ATRIBUTOS DO NÓ XML
    // =========================================================================
    /// Identificador exclusivo da assinatura se aplicável.
    #[serde(rename = "@Id", default)]
    pub id: Option<String>,

    // =========================================================================
    // 2. TEXTO E CAMPOS OPCIONAIS
    // =========================================================================
    /// Conteúdo de texto bruto se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    // =========================================================================
    // 3. CAMPOS ESTREITAMENTE OBRIGATÓRIOS
    // =========================================================================
    /// Informações sobre a chave pública e certificados do protocolo.
    #[serde(rename = "KeyInfo")]
    pub key_info: ProtSignatureXdKeyInfo,

    /// Valor criptográfico da assinatura do protocolo.
    #[serde(rename = "SignatureValue")]
    pub signature_value: ProtSignatureXdSignatureValue,

    /// Informações cobertas pela assinatura do protocolo.
    #[serde(rename = "SignedInfo")]
    pub signed_info: ProtSignatureXdSignedInfo,
}

/// Bloco de Chave Pública para Assinatura do Protocolo (`<KeyInfo>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProtSignatureXdKeyInfo {
    // =========================================================================
    // 1. ATRIBUTOS DO NÓ XML
    // =========================================================================
    /// Identificador exclusivo se aplicável.
    #[serde(rename = "@Id", default)]
    pub id: Option<String>,

    // =========================================================================
    // 2. TEXTO E CAMPOS OPCIONAIS
    // =========================================================================
    /// Conteúdo de texto se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    // =========================================================================
    // 3. CAMPOS ESTREITAMENTE OBRIGATÓRIOS
    // =========================================================================
    /// Dados correspondentes ao certificado X.509 do protocolo.
    #[serde(rename = "X509Data")]
    pub x509_data: ProtSignatureXdKeyInfoXdX509Data,
}

/// Bloco de Dados do Certificado X.509 da Assinatura do Protocolo (`<X509Data>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProtSignatureXdKeyInfoXdX509Data {
    /// Conteúdo de texto se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    /// Certificado digital correspondente em formato Base64.
    #[serde(rename = "X509Certificate", default)]
    pub x509_certificate: Option<String>,
}

/// Bloco de Valor da Assinatura do Protocolo (`<SignatureValue>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProtSignatureXdSignatureValue {
    /// Identificador exclusivo se aplicável.
    #[serde(rename = "@Id", default)]
    pub id: Option<String>,

    /// Conteúdo bruto em Base64 correspondente ao valor criptográfico da assinatura.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

/// Bloco de Informações Cobertas pela Assinatura do Protocolo (`<SignedInfo>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProtSignatureXdSignedInfo {
    // =========================================================================
    // 1. ATRIBUTOS DO NÓ XML
    // =========================================================================
    /// Identificador exclusivo se aplicável.
    #[serde(rename = "@Id", default)]
    pub id: Option<String>,

    // =========================================================================
    // 2. TEXTO E CAMPOS OPCIONAIS
    // =========================================================================
    /// Conteúdo de texto se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    // =========================================================================
    // 3. CAMPOS ESTREITAMENTE OBRIGATÓRIOS
    // =========================================================================
    /// Método de canonização XML adotado para o protocolo.
    #[serde(rename = "CanonicalizationMethod")]
    pub canonicalization_method: ProtSignatureXdSignedInfoXdCanonicalizationMethod,

    /// Referência de vinculação ao protocolo de autorização assinado.
    #[serde(rename = "Reference")]
    pub reference: ProtSignatureXdSignedInfoXdReference,

    /// Algoritmo criptográfico de assinatura adotado.
    #[serde(rename = "SignatureMethod")]
    pub signature_method: ProtSignatureXdSignedInfoXdSignatureMethod,
}

/// Método de Canonização XML da Assinatura do Protocolo (`<CanonicalizationMethod>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProtSignatureXdSignedInfoXdCanonicalizationMethod {
    /// URI correspondente ao algoritmo de canonização adotado.
    #[serde(rename = "@Algorithm", default)]
    pub algorithm: Option<String>,
}

/// Referência de Vinculação da Assinatura do Protocolo (`<Reference>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProtSignatureXdSignedInfoXdReference {
    // =========================================================================
    // 1. ATRIBUTOS DO NÓ XML
    // =========================================================================
    /// Identificador exclusivo se aplicável.
    #[serde(rename = "@Id", default)]
    pub id: Option<String>,

    /// Tipo de referência adotada se aplicável.
    #[serde(rename = "@Type", default)]
    pub reference_type: Option<String>,

    /// URI de referência ao nó assinado.
    #[serde(rename = "@URI", default)]
    pub uri: Option<String>,

    // =========================================================================
    // 2. TEXTO E CAMPOS OPCIONAIS
    // =========================================================================
    /// Conteúdo de texto bruto se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    /// Valor do resumo digital (hash) calculado para o protocolo.
    #[serde(rename = "DigestValue", default)]
    pub digest_value: Option<String>,

    // =========================================================================
    // 3. CAMPOS ESTREITAMENTE OBRIGATÓRIOS
    // =========================================================================
    /// Algoritmo de resumo digital adotado para gerar o hash.
    #[serde(rename = "DigestMethod")]
    pub digest_method: ProtSignatureXdSignedInfoXdReferenceXdDigestMethod,

    /// Conjunto de transformações aplicadas.
    #[serde(rename = "Transforms")]
    pub transforms: ProtSignatureXdSignedInfoXdReferenceXdTransforms,
}

/// Algoritmo de Resumo Digital da Assinatura do Protocolo (`<DigestMethod>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProtSignatureXdSignedInfoXdReferenceXdDigestMethod {
    /// URI correspondente ao algoritmo de resumo digital adotado.
    #[serde(rename = "@Algorithm", default)]
    pub algorithm: Option<String>,
}

/// Bloco de Transformações XML da Assinatura do Protocolo (`<Transforms>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProtSignatureXdSignedInfoXdReferenceXdTransforms {
    /// Conteúdo de texto se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    /// Lista de transformações aplicadas ao protocolo.
    #[serde(rename = "Transform", default)]
    pub transform: Vec<ProtSignatureXdSignedInfoXdReferenceXdTransformsXdTransform>,
}

/// Detalhes de Transformação da Assinatura do Protocolo (`<Transform>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProtSignatureXdSignedInfoXdReferenceXdTransformsXdTransform {
    /// URI correspondente ao algoritmo de transformação adotado.
    #[serde(rename = "@Algorithm", default)]
    pub algorithm: Option<String>,

    /// Conteúdo de texto se aplicável.
    #[serde(rename = "$text", default)]
    pub text: Option<String>,

    /// Expressão XPath de corte se aplicável.
    #[serde(rename = "XPath", default)]
    pub xpath: Option<String>,
}

/// Algoritmo de Assinatura Criptográfica da Assinatura do Protocolo (`<SignatureMethod>`).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProtSignatureXdSignedInfoXdSignatureMethod {
    /// URI correspondente ao algoritmo de assinatura adotado.
    #[serde(rename = "@Algorithm", default)]
    pub algorithm: Option<String>,
}

use serde::{Deserialize, Serialize};

// Signature

#[derive(Debug, Serialize, Deserialize)]
pub struct Signature {
    #[serde(rename = "@xmlns")]
    pub xmlns: Option<String>,
    #[serde(rename = "@xsi:schemaLocation")]
    pub xsi_schema_location: Option<String>,
    #[serde(rename = "@Id")]
    pub id: Option<String>,
    #[serde(rename = "@xmlns:xsi")]
    pub xmlns_xsi: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "SignedInfo")]
    pub signed_info: SignedInfo,
    #[serde(rename = "SignatureValue")]
    pub signature_value: SignatureValue,
    #[serde(rename = "KeyInfo")]
    pub key_info: KeyInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SignedInfo {
    #[serde(rename = "@Id")]
    pub id: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "CanonicalizationMethod")]
    pub canonicalization_method: CanonicalizationMethod,
    #[serde(rename = "SignatureMethod")]
    pub signature_method: SignatureMethod,
    #[serde(rename = "Reference")]
    pub reference: Reference,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CanonicalizationMethod {
    #[serde(rename = "@Algorithm")]
    pub algorithm: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SignatureMethod {
    #[serde(rename = "@Algorithm")]
    pub algorithm: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Reference {
    #[serde(rename = "DigestValue")]
    pub digest_value: Option<String>,
    #[serde(rename = "@Id")]
    pub id: Option<String>,
    #[serde(rename = "@URI")]
    pub uri: Option<String>,
    #[serde(rename = "@Type")]
    pub reference_type: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "Transforms")]
    pub transforms: Transforms,
    #[serde(rename = "DigestMethod")]
    pub digest_method: DigestMethod,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Transforms {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "Transform")]
    pub transform: Vec<Transform>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Transform {
    #[serde(rename = "XPath")]
    pub xpath: Option<Vec<String>>,
    #[serde(rename = "@Algorithm")]
    pub algorithm: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DigestMethod {
    #[serde(rename = "@Algorithm")]
    pub algorithm: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SignatureValue {
    #[serde(rename = "@Id")]
    pub id: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KeyInfo {
    #[serde(rename = "@Id")]
    pub id: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "X509Data")]
    pub x509_data: X509Data,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct X509Data {
    #[serde(rename = "X509Certificate")]
    pub x509_certificate: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProtSignature {
    #[serde(rename = "@Id")]
    pub id: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "KeyInfo")]
    pub key_info: ProtSignatureXdKeyInfo,
    #[serde(rename = "SignatureValue")]
    pub signature_value: ProtSignatureXdSignatureValue,
    #[serde(rename = "SignedInfo")]
    pub signed_info: ProtSignatureXdSignedInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProtSignatureXdKeyInfo {
    #[serde(rename = "@Id")]
    pub id: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "X509Data")]
    pub x509_data: ProtSignatureXdKeyInfoXdX509Data,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProtSignatureXdKeyInfoXdX509Data {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "X509Certificate")]
    pub x509_certificate: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProtSignatureXdSignatureValue {
    #[serde(rename = "@Id")]
    pub id: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProtSignatureXdSignedInfo {
    #[serde(rename = "@Id")]
    pub id: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "CanonicalizationMethod")]
    pub canonicalization_method: ProtSignatureXdSignedInfoXdCanonicalizationMethod,
    #[serde(rename = "Reference")]
    pub reference: ProtSignatureXdSignedInfoXdReference,
    #[serde(rename = "SignatureMethod")]
    pub signature_method: ProtSignatureXdSignedInfoXdSignatureMethod,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProtSignatureXdSignedInfoXdCanonicalizationMethod {
    #[serde(rename = "@Algorithm")]
    pub algorithm: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProtSignatureXdSignedInfoXdReference {
    #[serde(rename = "@Id")]
    pub id: Option<String>,
    #[serde(rename = "@Type")]
    pub reference_type: Option<String>,
    #[serde(rename = "@URI")]
    pub uri: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "DigestMethod")]
    pub digest_method: ProtSignatureXdSignedInfoXdReferenceXdDigestMethod,
    #[serde(rename = "DigestValue")]
    pub digest_value: Option<String>,
    #[serde(rename = "Transforms")]
    pub transforms: ProtSignatureXdSignedInfoXdReferenceXdTransforms,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProtSignatureXdSignedInfoXdReferenceXdDigestMethod {
    #[serde(rename = "@Algorithm")]
    pub algorithm: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProtSignatureXdSignedInfoXdReferenceXdTransforms {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "Transform")]
    pub transform: Vec<ProtSignatureXdSignedInfoXdReferenceXdTransformsXdTransform>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProtSignatureXdSignedInfoXdReferenceXdTransformsXdTransform {
    #[serde(rename = "@Algorithm")]
    pub algorithm: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "XPath")]
    pub xpath: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProtSignatureXdSignedInfoXdSignatureMethod {
    #[serde(rename = "@Algorithm")]
    pub algorithm: Option<String>,
}

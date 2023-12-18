use serde::{Deserialize, Serialize};

// Signature

#[derive(Debug, Serialize, Deserialize)]
pub struct Signature {
    #[serde(rename = "@xmlns")]
    pub xmlns: String,
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
    pub algorithm: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SignatureMethod {
    #[serde(rename = "@Algorithm")]
    pub algorithm: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Reference {
    #[serde(rename = "DigestValue")]
    pub digest_value: String,
    #[serde(rename = "@Id")]
    pub id: Option<String>,
    #[serde(rename = "@URI")]
    pub uri: String,
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
    pub algorithm: String,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DigestMethod {
    #[serde(rename = "@Algorithm")]
    pub algorithm: String,
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
    pub x509_certificate: String,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

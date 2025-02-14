#![allow(clippy::upper_case_acronyms)]

use serde::{Deserialize, Serialize};

// Fonte:
// https://stackoverflow.com/questions/59452193/difficulties-deserializing-xml-using-rust-and-serde-where-document-has-optional

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct DEFECTS {
    #[serde(rename = "DEFECT", default)]
    pub defects: Vec<DEFECT>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct DEFECT {
    #[serde(default)]
    pub SFA: SFA,
    pub DEFECTCODE: String,
    pub DESCRIPTION: String,
    pub FUNCTION: String,
    pub DECORATED: String,
    pub FUNCLINE: String,
    pub PATH: PATH,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct PATH {
    pub SFA: Option<SFA>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct SFA {
    pub FILEPATH: String,
    pub FILENAME: String,
    pub LINE: String,
    pub COLUMN: String,
}

#[cfg(test)]
mod test_example01 {
    use super::*;
    use crate::MyResult;
    use quick_xml::de::from_reader;
    use std::io::{BufReader, Cursor};

    const XML: &str = r#"
<?xml version="1.0" encoding="utf-8"?>
<DEFECTS>
  <DEFECT>
    <SFA>
      <FILEPATH>c:\projects\source\repos\defecttest\defecttest</FILEPATH>
      <FILENAME>source.cpp</FILENAME>
      <LINE>8</LINE>
      <COLUMN>5</COLUMN>
    </SFA>
    <DEFECTCODE>26496</DEFECTCODE>
    <DESCRIPTION>The variable 'y' is assigned only once, mark it as const (con.4).</DESCRIPTION>
    <FUNCTION>main</FUNCTION>
    <DECORATED>main</DECORATED>
    <FUNCLINE>6</FUNCLINE>
    <PATH></PATH>
  </DEFECT>
  <DEFECT>
    <SFA>
      <FILEPATH>c:\projects\source\repos\defecttest\defecttest</FILEPATH>
      <FILENAME>source.cpp</FILENAME>
      <LINE>9</LINE>
      <COLUMN>5</COLUMN>
    </SFA>
    <DEFECTCODE>26496</DEFECTCODE>
    <DESCRIPTION>The variable 'z' is assigned only once, mark it as const (con.4).</DESCRIPTION>
    <FUNCTION>main</FUNCTION>
    <DECORATED>main</DECORATED>
    <FUNCLINE>6</FUNCLINE>
    <PATH>
      <SFA>
        <FILEPATH>c:\projects\source\repos\defecttest\defecttest</FILEPATH>
        <FILENAME>source.cpp</FILENAME>
        <LINE>12</LINE>
        <COLUMN>3</COLUMN>
      </SFA>
    </PATH>
  </DEFECT>
</DEFECTS>"#;

    #[test]
    /// `cargo test -- --show-output deserialize_xml_defects`
    fn deserialize_xml_defects() -> MyResult<()> {
        // Create fake "file"
        let cursor = Cursor::new(XML);

        // Open the file in read-only mode.
        // let f = File::open("xml_path")?;

        let mut reader = BufReader::new(cursor);

        // Now, try to deserialize the XML we have in file_content
        let defect_list: DEFECTS = from_reader(&mut reader)?;

        // Assuming the unwrap above didn't blow up, we should get a count here
        println!("defect_list.defects.len(): {}", defect_list.defects.len());

        println!("defect_list: {:#?}", defect_list);

        //assert_eq!(valid, results);

        Ok(())
    }
}

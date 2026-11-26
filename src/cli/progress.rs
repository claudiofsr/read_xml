//! # Gerenciamento de Barras de Progresso
//!
//! Encapsula a renderização multi-thread das barras de progresso via [`indicatif`].

use crate::XmlParserResult;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

#[derive(Debug)]
pub struct MultiProgressBar {
    pub show_parse: ProgressBar,
    pub show_print: ProgressBar,
    pub show_excel: ProgressBar,
    pub show_csval: ProgressBar,
}

impl Default for MultiProgressBar {
    fn default() -> Self {
        Self {
            show_parse: ProgressBar::hidden(),
            show_print: ProgressBar::hidden(),
            show_excel: ProgressBar::hidden(),
            show_csval: ProgressBar::hidden(),
        }
    }
}

impl MultiProgressBar {
    fn add_bar(
        &mut self,
        multi_progress: &MultiProgress,
        prefix: &str,
        total: usize,
    ) -> XmlParserResult<ProgressBar> {
        let nchar = total.to_string().chars().count();
        let template = format!(
            "{{prefix:<10.bold.dim}} {{spinner:.green}} [{{elapsed_precise}}] [{{wide_bar:.cyan/blue}}] {{pos:>{nchar}}}/{{len:>{nchar}}} ({{eta}})"
        );

        let style = ProgressStyle::default_bar()
            .template(&template)?
            .progress_chars("#>-");

        let pb = multi_progress.add(ProgressBar::new(total as u64));
        pb.set_style(style);
        pb.set_prefix(prefix.to_string());

        Ok(pb)
    }

    pub fn add_parse_xml(&mut self, mp: &MultiProgress, total: usize) -> XmlParserResult<()> {
        self.show_parse = self.add_bar(mp, "parse xml", total)?;
        Ok(())
    }

    pub fn add_print_xml(&mut self, mp: &MultiProgress, total: usize) -> XmlParserResult<()> {
        self.show_print = self.add_bar(mp, "print xml", total)?;
        Ok(())
    }

    pub fn add_print_xls(&mut self, mp: &MultiProgress, total: usize) -> XmlParserResult<()> {
        self.show_excel = self.add_bar(mp, "write xlsx", total)?;
        Ok(())
    }

    pub fn add_print_csv(&mut self, mp: &MultiProgress, total: usize) -> XmlParserResult<()> {
        self.show_csval = self.add_bar(mp, "write csv", total)?;
        Ok(())
    }
}

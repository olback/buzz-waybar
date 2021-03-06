use {serde::Serialize, std::collections::HashMap};

#[derive(Debug, Serialize)]
pub struct ReturnData {
    pub text: String,
    pub alt: String,
    pub percentage: u8,
    pub tooltip: String,
}

impl ReturnData {
    pub fn new(data: &HashMap<String, Result<usize, String>>) -> Self {
        let n: usize = data.iter().map(|v| v.1.as_ref().unwrap_or(&0)).sum();
        Self {
            text: format!("{} unread", n),
            alt: n.to_string(),
            percentage: if n > 0 { 100 } else { 0 },
            tooltip: {
                let mut tooltip_lines = Vec::<String>::with_capacity(data.len() + 1);
                tooltip_lines.push(format!(
                    "<b>You have {} unread {}</b>",
                    n,
                    if n > 1 { "emails" } else { "email" }
                ));
                for (name, unread) in data {
                    tooltip_lines.push(format!(
                        "{}: {}",
                        name,
                        match unread {
                            Ok(n) => n.to_string(),
                            Err(e) => e.clone(),
                        }
                    ));
                }
                tooltip_lines
            }
            .join("\n"),
        }
    }
}

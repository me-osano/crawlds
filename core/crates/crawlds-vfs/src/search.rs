//! Search module - Tantivy-based full-text search

use crate::error::VfsError;
use crate::fs::Entry;
use std::path::Path;
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::*;
use tantivy::{Index, IndexWriter, ReloadPolicy, TantivyDocument};

#[allow(dead_code)]
pub struct SearchEngine {
    index: Index,
    writer: IndexWriter,
    schema: Schema,
    fields: SearchFields,
}

struct SearchFields {
    path: Field,
    name: Field,
    body: Field,
    ext: Field,
    mime: Field,
    size: Field,
    mtime: Field,
}

impl SearchEngine {
    pub fn new(index_path: &Path) -> Result<Self, VfsError> {
        let mut schema_builder = Schema::builder();

        let path = schema_builder.add_text_field("path", STRING | STORED);
        let name = schema_builder.add_text_field("name", TEXT | STORED);
        let body = schema_builder.add_text_field("body", TEXT);
        let ext = schema_builder.add_text_field("ext", STRING | STORED);
        let mime = schema_builder.add_text_field("mime", STRING | STORED);
        let size = schema_builder.add_u64_field("size", INDEXED | STORED);
        let mtime = schema_builder.add_i64_field("mtime", INDEXED | STORED);

        let schema = schema_builder.build();

        std::fs::create_dir_all(index_path).map_err(|e| VfsError::IndexError(e.to_string()))?;

        let index = Index::create_in_dir(index_path, schema.clone())
            .or_else(|_| Index::open_in_dir(index_path))
            .map_err(|e| VfsError::IndexError(e.to_string()))?;

        let writer = index
            .writer(50_000_000)
            .map_err(|e| VfsError::IndexError(e.to_string()))?;

        let fields = SearchFields {
            path,
            name,
            body,
            ext,
            mime,
            size,
            mtime,
        };

        Ok(Self {
            index,
            writer,
            schema,
            fields,
        })
    }

    pub fn index_entry(&mut self, entry: &Entry) -> Result<(), VfsError> {
        let mut doc = TantivyDocument::default();

        doc.add_text(self.fields.path, entry.path.to_string_lossy());
        doc.add_text(self.fields.name, entry.name.clone());
        doc.add_text(self.fields.ext, entry.extension.clone());
        doc.add_text(self.fields.mime, entry.mime.clone());
        doc.add_u64(self.fields.size, entry.size);
        doc.add_i64(
            self.fields.mtime,
            entry
                .mtime
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0),
        );

        self.writer
            .add_document(doc)
            .map_err(|e| VfsError::IndexError(e.to_string()))?;

        Ok(())
    }

    pub fn commit(&mut self) -> Result<(), VfsError> {
        self.writer
            .commit()
            .map_err(|e| VfsError::IndexError(e.to_string()))?;
        Ok(())
    }

    pub fn remove_path(&mut self, path: &str) -> Result<(), VfsError> {
        let term = tantivy::Term::from_field_text(self.fields.path, path);
        self.writer.delete_term(term);
        Ok(())
    }

    pub fn search(
        &self,
        query_str: &str,
        folder: Option<&str>,
        limit: usize,
    ) -> Result<Vec<SearchHit>, VfsError> {
        let reader = self
            .index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommitWithDelay)
            .try_into()
            .map_err(|e: tantivy::TantivyError| VfsError::IndexError(e.to_string()))?;

        let searcher = reader.searcher();

        let query_parser =
            QueryParser::for_index(&self.index, vec![self.fields.name, self.fields.body]);

        let query = query_parser
            .parse_query(query_str)
            .map_err(|e| VfsError::SearchFailed(e.to_string()))?;

        let top_docs = searcher
            .search(&query, &TopDocs::with_limit(limit))
            .map_err(|e| VfsError::SearchFailed(e.to_string()))?;

        let mut hits = Vec::new();
        for (_score, doc_address) in top_docs {
            if let Ok(retrieved_doc) = searcher.doc::<TantivyDocument>(doc_address) {
                let path = retrieved_doc
                    .get_first(self.fields.path)
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let name = retrieved_doc
                    .get_first(self.fields.name)
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let ext = retrieved_doc
                    .get_first(self.fields.ext)
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let mime = retrieved_doc
                    .get_first(self.fields.mime)
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let size = retrieved_doc
                    .get_first(self.fields.size)
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                let mtime = retrieved_doc
                    .get_first(self.fields.mtime)
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0);

                if let Some(folder) = folder {
                    if !path.starts_with(folder) {
                        continue;
                    }
                }

                hits.push(SearchHit {
                    path,
                    name,
                    ext,
                    mime,
                    size,
                    mtime,
                });
            }
        }

        Ok(hits)
    }

    pub fn clear(&mut self) -> Result<(), VfsError> {
        self.writer
            .delete_all_documents()
            .map_err(|e| VfsError::IndexError(e.to_string()))?;
        self.writer
            .commit()
            .map_err(|e| VfsError::IndexError(e.to_string()))?;
        Ok(())
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SearchHit {
    pub path: String,
    pub name: String,
    pub ext: String,
    pub mime: String,
    pub size: u64,
    pub mtime: i64,
}

// ── Convenience functions ───────────────────────────────────────────────────

pub fn get_default_index_path() -> Option<std::path::PathBuf> {
    std::env::var("XDG_DATA_HOME")
        .ok()
        .map(std::path::PathBuf::from)
        .or_else(|| std::env::var("HOME").ok().map(|h| std::path::PathBuf::from(h).join(".local/share")))
        .map(|p| p.join("crawlds").join("search_index"))
}

pub async fn search(root: &str, query: &str, max_results: usize) -> Result<Vec<SearchHit>, VfsError> {
    let index_path = get_default_index_path().ok_or_else(|| VfsError::IndexError("no index path".to_string()))?;
    let engine = SearchEngine::new(&index_path)?;
    engine.search(query, Some(root), max_results)
}

pub async fn search_home(query: &str, max_results: usize) -> Result<Vec<SearchHit>, VfsError> {
    let home = std::env::var("HOME").map_err(|_| VfsError::IndexError("no home".to_string()))?;
    search(&home, query, max_results).await
}

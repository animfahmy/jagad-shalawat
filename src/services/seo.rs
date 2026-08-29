#![allow(dead_code)]
use crate::models::post::BlogPost;

pub fn generate_json_ld_article(post: &BlogPost, base_url: &str, lang: &str) -> String {
    let url = if lang == "en" {
        format!("{}/blog/en/{}", base_url, post.slug_en.as_deref().unwrap_or(&post.slug))
    } else {
        format!("{}/blog/{}", base_url, post.slug)
    };
    
    let image_field = if let Some(img) = &post.featured_image {
        format!(r#""image": "{}","#, img)
    } else {
        String::new()
    };
    
    let description = if let Some(desc) = &post.excerpt {
        desc
    } else {
        &post.title
    };

    let is_based_on_field = if let Some(sources_val) = &post.sources {
        if let Some(arr) = sources_val.as_array() {
            let urls: Vec<String> = arr.iter().filter_map(|item| {
                if let Some(obj) = item.as_object() {
                    obj.get("url").and_then(|u| u.as_str()).map(|s| format!("\"{}\"", s))
                } else if let Some(s) = item.as_str() {
                    Some(format!("\"{}\"", s))
                } else {
                    None
                }
            }).collect();
            if !urls.is_empty() {
                format!(r#""isBasedOn": [{}],"#, urls.join(","))
            } else {
                String::new()
            }
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    let pub_date = post.published_at.map(|d| d.format("%Y-%m-%dT%H:%M:%SZ").to_string()).unwrap_or_default();
    let mod_date = post.updated_at.format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let author_name = if !post.author_name.is_empty() {
        &post.author_name
    } else {
        post.source_name.as_deref().unwrap_or("Tim Jagad Shalawat")
    };
    
    let json = format!(r#"<script type="application/ld+json">
{{
    "@context": "https://schema.org",
    "@type": "Article",
    "headline": "{}",
    {}
    {}
    "datePublished": "{}",
    "dateModified": "{}",
    "author": {{
        "@type": "Person",
        "name": "{}"
    }},
    "description": "{}",
    "url": "{}"
}}
</script>"#, 
    post.title.replace('"', "\\\""), 
    image_field,
    is_based_on_field, 
    pub_date, 
    mod_date, 
    author_name.replace('"', "\\\""), 
    description.replace('"', "\\\""), 
    url);
    
    json
}

pub fn generate_json_ld_breadcrumb(items: &[(String, String)]) -> String {
    let mut list_items = Vec::new();
    
    for (i, (name, url)) in items.iter().enumerate() {
        let item = format!(r#"{{
            "@type": "ListItem",
            "position": {},
            "name": "{}",
            "item": "{}"
        }}"#, i + 1, name.replace('"', "\\\""), url);
        list_items.push(item);
    }
    
    format!(r#"<script type="application/ld+json">
{{
    "@context": "https://schema.org",
    "@type": "BreadcrumbList",
    "itemListElement": [{}]
}}
</script>"#, list_items.join(","))
}

pub fn generate_og_tags(title: &str, description: &str, image: Option<&str>, url: &str, locale: &str) -> String {
    let mut tags = format!(
        r#"<meta property="og:title" content="{}" />
<meta property="og:description" content="{}" />
<meta property="og:url" content="{}" />
<meta property="og:locale" content="{}" />
<meta property="og:type" content="article" />"#,
        title.replace('"', "&quot;"),
        description.replace('"', "&quot;"),
        url,
        locale
    );

    if let Some(img) = image {
        tags.push_str(&format!("\n<meta property=\"og:image\" content=\"{}\" />", img));
    }

    tags
}

pub fn generate_hreflang_tags(slug_id: &str, slug_en: Option<&str>, base_url: &str) -> String {
    let id_url = format!("{}/blog/{}", base_url, slug_id);
    let mut tags = format!(r#"<link rel="alternate" hreflang="id" href="{}" />"#, id_url);
    
    if let Some(en) = slug_en {
        let en_url = format!("{}/blog/en/{}", base_url, en);
        tags.push_str(&format!("\n<link rel=\"alternate\" hreflang=\"en\" href=\"{}\" />", en_url));
        tags.push_str(&format!("\n<link rel=\"alternate\" hreflang=\"x-default\" href=\"{}\" />", id_url));
    } else {
        tags.push_str(&format!("\n<link rel=\"alternate\" hreflang=\"x-default\" href=\"{}\" />", id_url));
    }

    tags
}

pub fn generate_canonical(url: &str) -> String {
    format!(r#"<link rel="canonical" href="{}" />"#, url)
}


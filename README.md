# Doxcer

Een kleine Rust-tool ontwikkeld door **Stefan B. J. Meeuwessen** voor het automatisch genereren van documentatie (in Markdown) van **Fabric PySpark notebooks**.  
Het script analyseert een Python-notebook en genereert op basis van een vaste template een gestructureerd document met **functionele** en **technische** tabellen.

---

## 📋 Functioneel overzicht

**Doel:**  
Doxcer helpt bij het standaardiseren van documentatie voor data pipelines binnen Fabric.  
Door de inhoud van een PySpark-notebook te analyseren, wordt automatisch een Markdown-document aangemaakt met:

- **Functionele documentatie:** Voor BI-experts — beschrijft kolommen, betekenis en logische voorwaarden.  
- **Technische documentatie:** Voor Data Engineers — beschrijft datatypes, bronnen, joins, transformaties en ETL-opbouw.

Het resultaat volgt deze opbouw:

```yaml
---
author: StefanGPT
notebook: <notebook_naam.py>
created: <ISO datetime>
---
```

```Markdown
# Notebook omschrijving
{beschrijf hier kort wat dit notebook doet}

---

## Functioneel ontwerp

| **Attribuut naam**        | **Definitie**                              | **Omschrijving transformatie**                                                                                      |
| ------------------------- | ------------------------------------------ | ------------------------------------------------------------------------------------------------------------------- |
| voorbeeld: dim_project_fk | voorbeeld: de foreign key naar dim_project | voorbeeld: De SK van dim_project_t wordt gepakt en ge-aliast naar dim_project_fk. De data blijft verder het zelfde. |
|                           |                                            |                                                                                                                     |

---

## Technisch ontwerp

| **Atribuut naam**         | **Data Type**     | **Key**       | **Bron**                   | **Brontabel(en)**             | **Bronattribuut(en)**     | **Voordaarde**                                                                     |
| ------------------------- | ----------------- | ------------- | -------------------------- | ----------------------------- | ------------------------- | ---------------------------------------------------------------------------------- |
| voorbeeld: dim_project_fk | voorbeeld: string | voorbeeld: Ja | voorbeeld: Staff-Lakehouse | voorbeeld: gold.dim_project_t | voorbeeld: dim_project_sk | voorbeeld: F.col("dim_project_sk").cast("string").alias("dim_project_fk"),         |
|                           |                   |               |                            |                               |                           |                                                                                    |
```

## ⚙️ How to Use
1. Voorbereiding
Zorg dat je ``.env`` bestand aanwezig is in ``./config/.env`` met de volgende variabelen:
```.env
ENCRYPTION_PASSWORD=<je_fernet_sleutel>
OPENAI_API_KEY_ENC=<versleutelde_api_key>
```
De ``OPENAI_API_KEY_ENC`` is de met Fernet versleutelde API-sleutel van OpenAI.
De tool gebruikt deze sleutel om beveiligd te communiceren met het GPT-model.

---

2. CLI gebruik
Voer de tool uit via de command line:
```Shell
doxcer 'path/to/notebook.py'
```

Voorbeeld:
```Shell
doxcer ./fabric/gold/dim_project_t.py
```

De tool zal:
1. Het .env-bestand laden en de API-sleutel ontsleutelen.
2. Het opgegeven PySpark-notebook inlezen.
3. Een GPT-aanvraag doen naar gpt-5-mini.
4. De gegenereerde Markdown-documentatie direct printen in de console.

Wil je de output opslaan als bestand?
Gebruik dan:
```Shell
doxcer ./fabric/gold/dim_project_t.py > ./docs/dim_project_t.md
```

---

🧠 Architectuur
- dotenvy → Laadt configuratie uit .env
- fernet → Versleutelt en ontsleutelt de OpenAI API key
- reqwest → Verstuurd de API-aanroep naar OpenAI
- serde / serde_json → Voor (de)serialisatie van JSON-data
- std::fs / env / process → Bestand- en argumentbeheer

---

```Markdown
📦 Projectstructuur
project-root/
│
├── src/
│   └── main.rs
│
├── config/
│   └── .env
│
├── docs/
│   └── <gegenereerde_md_bestanden>
│
└── Cargo.toml
```

---

### Credits

~ Dhr. Stefan B. J. Meeuwessen ~
Analyzeer deze fabric pyspark notebook en schrijf in Markdown documentatie:

1. De opgeleverde dim of fact in twee tabellen;
    - Functioneel (Voor de BI experts, kolommen, inhoud van deze kolommen en functionele voorwaarden van de tabel)
    - Technisch (Voor de DData Engineers, hoe werkt de tabel en hoe hebben we de ETL kolommen technisch opgezet?)
2. Zorg er voor dat de tabellen als markdown tables worden gegenereerd.
3. Houd deze YAML en Markdown template aan:

```Markdown
---
author: StefanGPT
notebook: {notebook name here}
created: {creation data iso datetime}
---

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
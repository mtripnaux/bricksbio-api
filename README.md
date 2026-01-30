## Providers

 - [`iGEM Parts Registry (Legacy)`](https://parts.igem.org/) is a HTML scraper, making 2 requests.
 - [`iGEM Registry`](https://registry.igem.org) is a single JSON request.
 - [`iGEM via SynBioHub`](https://synbiohub.org/public/igem/igem_collection/1) is a SBOL and GB parser, making 2 requests.
  - [`Ensembl`](https://www.ensembl.org) is a GB parser, making 1 request.

## Schema

 - `type` represents the type of a sequence.
    - `canonical` is a standardized custom format.
    - `ontology` is the [Sequence Ontology](http://sequenceontology.org/) identifier.
    - `css` is the type's class name in [SBOL Visual CSS](https://edinburgh-genome-foundry.github.io/SBOL-Visual-CSS/).
    - `slug` is the new iGEM standard for storing types.
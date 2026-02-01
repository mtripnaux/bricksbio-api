<div align="center">
  <img width="500" height="110" alt="image" src="https://github.com/user-attachments/assets/e1e6a8d2-1161-4109-b34a-cf5346438f88" />
  <p>
  Universal API for <b>Synthetic Biology</b> Parts</span><br/><br/>
  <img alt="Passing" src="https://img.shields.io/badge/build-passing-brightgreen">
  <img alt="GitHub Forks" src="https://img.shields.io/github/forks/mtripnaux/bricks-api">
  <img alt="GitHub Repo stars" src="https://img.shields.io/github/stars/mtripnaux/bricks-api">
  </p>
  <p>
	   <a href="https://github.com/mtripnaux/bricks-api?tab=readme-ov-file#usage">Usage</a> - 
	   <a href="https://github.com/mtripnaux/bricks-api?tab=readme-ov-file#providers">Providers</a> - 
    <a href="https://github.com/mtripnaux/bricks-api?tab=readme-ov-file#schema">Schema</a> 
  </p>
</div>

## Usage

To use this API, you can directly request [bricks.bio](https://bricks.bio/api/). Since everything is parsed or scraped from online public resources, you can also self-host this API. However, your local version might be slower at first, due to the fact that we use [response caching](https://restfulapi.net/caching/) of biobricks files. The API request template is extremely simple, you can either ask for a single part using its unique ID (often given by the provider), or perform a meta-search trough all cached biobricks. If you wish to run on local, you will have to use a pre-caching script in order to use the search feature.

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

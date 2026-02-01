<div align="center">
  <img width="500" height="110" alt="image" src="https://github.com/user-attachments/assets/e1e6a8d2-1161-4109-b34a-cf5346438f88" />
  <p>
  Universal API for <b>Synthetic Biology</b> Parts</span><br/><br/>
  <img alt="Passing" src="https://img.shields.io/badge/build-passing-brightgreen">
  <img alt="GitHub Forks" src="https://img.shields.io/github/forks/mtripnaux/bricksbio-api">
  <img alt="GitHub Repo stars" src="https://img.shields.io/github/stars/mtripnaux/bricksbio-api">
  </p>
  <p>
	   <a href="https://github.com/mtripnaux/bricksbio-api?tab=readme-ov-file#usage">Usage</a> - 
	   <a href="https://github.com/mtripnaux/bricksbio-api?tab=readme-ov-file#providers">Providers</a> - 
    <a href="https://github.com/mtripnaux/bricksbio-api?tab=readme-ov-file#schema">Schema</a> 
  </p>
</div>

## Introduction

Currently, the synthetic biology part repositories are disseminated, inconsistent and rarely maintained (see the [Providers](https://github.com/mtripnaux/bricksbio-api?tab=readme-ov-file#providers) section). Almost no programmatic interface exist for these databases, despite endpoints to download hand-written, incomplete or extremely heterogenous data. We provide a simple, [RESTful](https://restfulapi.net/) interface to centralize this field and help accelerate its evolution, especially through large-scale automation.

In addition to that, different standards are co-existing for synbio data storage ([SBOL](https://sbolstandard.org/), [GenBank](https://www.ncbi.nlm.nih.gov/genbank/) and others) but none of them is actually efficient. It either includes relevant data for genomic studies, or provides insights about the authorship, or describes the sequence's features. We introduce the Biobrick JSON schema, in tribute to the (now retired) [BioBricks Foundation (WebArchive)](https://web.archive.org/web/20120503070441/https://biobricks.org/) notably behind the [BioBrick Assembly Standard](https://en.wikipedia.org/wiki/BioBrick).

> The Biobrick.json format is not intended to replace SBOL, but to simplify the querying and parsing of synthetic biology data. Every file from the Bricks.bio API can be exported to SBOL at any time.

## Usage

To use this API, you can directly request [bricks.bio](https://bricks.bio/). Since everything is parsed or scraped from online public resources, you can also self-host this API. However, your local version might be slower at first, due to the fact that we use [response caching](https://restfulapi.net/caching/) of biobricks files. The API request template is extremely simple, you can either ask for a single part using its unique ID (often given by the provider), or perform a meta-search trough all cached biobricks. If you wish to run on local, you will have to use a pre-caching script in order to use the search feature.

## Testing

A simple test script is located in `bench/providers.sh`. It essentially makes various queries for part IDs located on different providers, and outputs a table including the ID, the [response status](https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Status) and the response time in miliseconds.

## Providers

 - [`iGEM Parts Registry (Legacy)`](https://parts.igem.org/) is a HTML scraper, making 2 requests.
 - [`iGEM Registry`](https://registry.igem.org) is a single JSON request.
 - [`iGEM via SynBioHub`](https://synbiohub.org/public/igem/igem_collection/1) is a SBOL and GB parser, making 2 requests.
  - [`Ensembl`](https://www.ensembl.org) is a GB parser, making 1 request.
  - [`NCBI`](https://https://www.ncbi.nlm.nih.gov/) is a GB parser, making 1 request.

## Schema

The full Biobrick [JSON Schema](https://json-schema.org/) can be found under `docs/biobrick.schema.json`.

- `type` represents the type of a sequence.
    - `canonical` is a standardized custom format.
    - `ontology` is the [Sequence Ontology](http://sequenceontology.org/) identifier.
    - `css` is the type's class name in [SBOL Visual CSS](https://edinburgh-genome-foundry.github.io/SBOL-Visual-CSS/).
    - `slug` is the new iGEM standard for storing types.
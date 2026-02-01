use crate::types::Biobrick;

/// SBOL 2.0/3.0 simplistic exporter for Biobrick.

pub fn to_sbol_xml(biobrick: &Biobrick) -> String {
    let mut xml = String::new();
    xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    xml.push_str("<rdf:RDF xmlns:rdf=\"http://www.w3.org/1999/02/22-rdf-syntax-ns#\"\n");
    xml.push_str("         xmlns:sbol=\"http://sbols.org/v2#\"\n");
    xml.push_str("         xmlns:dcterms=\"http://purl.org/dc/terms/\">\n");

    let base_uri = "https://bricks.bio/sbol/";
    let part_uri = format!("{}{}", base_uri, biobrick.metadata.id);
    let seq_uri = format!("{}_seq", part_uri);

    // Component Definition
    xml.push_str(&format!("  <sbol:ComponentDefinition rdf:about=\"{}\">\n", part_uri));
    xml.push_str(&format!("    <dcterms:title>{}</dcterms:title>\n", escape_xml(&biobrick.metadata.name)));
    xml.push_str(&format!("    <dcterms:description>{}</dcterms:description>\n", escape_xml(&biobrick.metadata.description)));
    
    xml.push_str("    <sbol:type rdf:resource=\"http://www.biopax.org/release/biopax-level3.owl#DnaRegion\"/>\n");
    
    xml.push_str(&format!("    <sbol:role rdf:resource=\"http://identifiers.org/so/{}\"/>\n", 
        biobrick.metadata.r#type.ontology.as_deref().unwrap_or("SO:0000110")));

    xml.push_str(&format!("    <sbol:sequence rdf:resource=\"{}\"/>\n", seq_uri));

    for (i, feature) in biobrick.features.iter().enumerate() {
        let anno_uri = format!("{}/annotation_{}", part_uri, i);
        let range_uri = format!("{}/range_{}", anno_uri, i);
        
        xml.push_str("    <sbol:sequenceAnnotation>\n");
        xml.push_str(&format!("      <sbol:SequenceAnnotation rdf:about=\"{}\">\n", anno_uri));
        xml.push_str(&format!("        <dcterms:title>{}</dcterms:title>\n", escape_xml(&feature.name)));
        xml.push_str("        <sbol:location>\n");
        xml.push_str(&format!("          <sbol:Range rdf:about=\"{}\">\n", range_uri));
        xml.push_str(&format!("            <sbol:start>{}</sbol:start>\n", feature.location.start));
        xml.push_str(&format!("            <sbol:end>{}</sbol:end>\n", feature.location.end));
        xml.push_str(&format!("            <sbol:orientation rdf:resource=\"http://sbols.org/v2#{}\"/>\n", 
            if feature.location.forward { "inline" } else { "reverseComplement" }));
        xml.push_str("          </sbol:Range>\n");
        xml.push_str("        </sbol:location>\n");
        xml.push_str(&format!("        <sbol:role rdf:resource=\"http://identifiers.org/so/{}\"/>\n", 
            feature.r#type.ontology.as_deref().unwrap_or("SO:0000110")));
        xml.push_str("      </sbol:SequenceAnnotation>\n");
        xml.push_str("    </sbol:sequenceAnnotation>\n");
    }

    xml.push_str("  </sbol:ComponentDefinition>\n");

    // Sequence object
    xml.push_str(&format!("  <sbol:Sequence rdf:about=\"{}\">\n", seq_uri));
    xml.push_str(&format!("    <sbol:elements>{}</sbol:elements>\n", biobrick.sequence.to_lowercase()));
    xml.push_str("    <sbol:encoding rdf:resource=\"http://www.chem.qmul.ac.uk/iubmb/misc/naseq.html\"/>\n");
    xml.push_str("  </sbol:Sequence>\n");

    xml.push_str("</rdf:RDF>\n");
    xml
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
     .replace('<', "&lt;")
     .replace('>', "&gt;")
     .replace('"', "&quot;")
     .replace('\'', "&apos;")
}

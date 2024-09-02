We have following sets of model files:
- `xml-repo` - XML models collected in the Github repository in folder [Models](https://github.com/hallba/BioModelAnalyzer/tree/master/Models)
- `json-repo` - JSON models collected in the Github repository in folder [BackEndTests](https://github.com/hallba/BioModelAnalyzer/tree/master/src/BackEndTests)
- `json-export-from-repo` - we took models from `xml-repo` and `json-repo`, uploaded them to the [tool](https://biomodelanalyzer.org/tool.html), and exported them (so that we have the newest supported format)
- `json-export-from-tool` - selection of all models shown in the [tool](https://biomodelanalyzer.org/tool.html), exported in the newest JSON

Note that models can (by design) be replicated in different folders. 
We just use them to cover wide range of possible formats and model variants, so that we can properly test the parser.
In future, we should curate them a bit.

Note that model formats might not be the same. For example, the XML and JSON formats differ. Also, JSON models in `json-repo` sometimes differ in specific format.
We have following sets of model files:
- `xml-repo` - XML models collected in the Github repository in folder [Models](https://github.com/hallba/BioModelAnalyzer/tree/master/Models)
- `json-repo` - JSON models collected in the Github repository in folder [BackEndTests](https://github.com/hallba/BioModelAnalyzer/tree/master/src/BackEndTests)
- `json-export-from-repo` - we took models from `xml-repo` and `json-repo`, uploaded them to the [tool](https://biomodelanalyzer.org/tool.html), and exported them (so that we have the newest supported format)
- `json-export-from-tool` - selection of all models shown in the [tool](https://biomodelanalyzer.org/tool.html), exported in the newest JSON

For now, we have collected as many models as possible (from various sources) to cover a wide range of formats and variants, so that we can properly test the parser. 
We are doing this partially because the internal structure of the models is not always the same. 
For example, the general structures and field names in XML and JSON formats differ. 
Also, JSON models in `json-repo` sometimes use a different JSON structure than the ones the tool currently exports.

Note that this means that (by design) models can be replicated in different folders. 
In future, we should curate this repository a bit.

# 🌟 Motivation

training grammar based transformer on language structure and artificial rules

* input (sequence of tokens) -> (rule based inner state) -> output

1. find rules associated with tokens from the input
2. apply rules in current context
   1. search for more rules
   2. add new rules (for training)
   3. apply other rules
3. stop with final state

Example for learning a deep contextual structure of text strings

* find rules using first token
* find rules using same context
* use discovered rules to build new rules to represent local contextual structure

Example for cooking recipes

* learn basic vocabulary and language structure from general curriculum
* learn artificial annotations (recipe cost, i.e. ingredient prices, time cost, ...) from human annotations and artificial rules

1. sequentially find associated rules for all structural units (i.e. words, compounds, symbols)
   * parsing rules, matching rules
2. sequentially apply each rule in the given context, i.e.
   * price("5 tomatoes") => 5 \* price\_of(_Tomato_)
   * time\_cost("dice 3 onions") => time\_cost(_dicing, onion_) \* 3
   * time\_cost("1 teaspoon") => no rules found \~ 0?

Problems:

* natural language ambiguity (no necessary exact definition of text, no unambiguous parsing)
* quadratic space requirement of self-attention based networks

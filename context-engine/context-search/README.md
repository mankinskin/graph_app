

(21.12.24)

## Examples

```
let text = "helohelloehllo";
let model = ContextGraph::from(text);

...
let index = model.insert(text)?;
```
## Insert
```
let start = model.insert(text[0]);
let path = QueryPath::from(start);
let query_next = text[1..];

match
path.search_iter(model).find(query_next)
{
    Ok(index) => index,
    Err((path, err, cache)) =>
        model.join(path, cache)
}

```
## Find
```
let start = path.start();
let cache = Cache::new(model, path, query);
let states = cache.next_states(start);
bft(
    states,
    |state| cache.next_states(state)
)
```
## Join
```

```
## Structure


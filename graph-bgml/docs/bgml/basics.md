
### Meta And Maplet

The `meta` tag is where the metadata of a page goes, and contains only key-value pairs,
called `maplet`s.

```html
<meta>
    <maplet key="charset" value="UTF-8" />
    <maplet key="title">My Page</maplet>
    <maplet key="author" value="Maxine Zick <maxine@pnk.dev>" />
    <resource mime="image/x-icon" src="/favicon.ico" />
</meta>
```

Maplets can either be self-closing, in which case they need to be provided both a `key`
and a `value` attribute, or they can be containing elements, in which case you must provide
either a `key` or a `value` attribute, and the contents of the tag are treated as the attribute
that wasn't provided.

```html
<maplet key="title" value="My Page" />
<maplet key="title">My Page</maplet>
<maplet value="My Page">title</maplet>
```

The last example there with `title` as the contents doesn't make much sense for metadata,
but it makes maplets more flexible and general in their use. For example, you could use them
in a `select` tag as a canonical key-value pair, with the `key` being the actual value, and
the `value` being what gets displayed.

### Prelude

The `prelude` tag is similar to HTML's `head` tag in that it's where you put important
3rd-party scripts, style tags, metadata, links to css files, etc. Scripts are fetched but
not executed until after the prelude tag is closed. Then the scripts are run in the order
they're defined in, and must finish executing before the rest of the page content is loaded.

```html
<prelude>
    <resource mime="image/x-icon" src="/favicon.ico" />
    <resource mime="text/css" src="/index.css" />
    <resource mime="text/x-lua" src="index.lua" defer />
</prelude>
```

### Resource

The `resource` tag is the grid's counterpart to HTML's `link` tag.

```html
<prelude>
    <resource mime="image/x-icon" src="favicon.ico" />
    <resource mime="text/css" src="/index.css" />
    <resource mime="text/x-lua" src="index.lua" defer />
    <resource name="wasm.os" mime="application/wasm" src="/os.wasm" />
</prelude>
```

Resources expect 2 attributes minimum:
- mime: The MIME type of the resource
- src: The location of the resource

Resources also have some optional attributes that can be provided:
- name: named resources are able to referred to using their name in either runtime code
  or by other BGML tags.
- defer: defer is an attribute used for deferring the execution of a script until after
  content is loaded, rather than running immediately or running at the end of the prelude
  tag.

### Postlude

Much like the `prelude` tag, the `postlude` tag is a mirror to the `prelude` tag in that
it controls content load priority and ordering. However, postlude tag contents are not loaded
until after all prior content is fully loaded (and any scripts have finished executing). For
this reason, scripts not found in the postlude section should be only library code or
initializing event-hooks.

Any scripts marked with `defer` are run before the postlude is run, as it is assumed that
they are necessary for scripts located in the postlude section.

```html
<meta>
    <maplet key="charset" value="UTF-8" />
    <maplet key="title">My Page</maplet>
    <maplet key="author" value="Maxine Zick <maxine@pnk.dev>" />
</meta>
<prelude>
    <resource mime="image/x-icon" src="/favicon.ico" />
    <resource mime="text/css" src="/index.css" />
    <resource mime="text/x-lua" src="index.lua" defer />
    <resource name="wasm.os" mime="application/wasm" src="/os.wasm" />
</prelude>
<content>
    <script lang="lua">
        events.add_listener("load", function() console.log("loaded!") end)
        runtime.wasm.add_library("os", resource.get("wasm.os"))
    </script>
</content>
<postlude>
    <script lang="lua">
        console.log("hit postlude!")
        console.log(os.time()) -- loaded from "wasm.os" asset
    </script>
</postlude>
```

### Script

Unlike HTML's `script` tag, BGML's script tag is ALWAYS for inline code; if you're fetching
code then you're using the `resource` tag. As such, we know that the MIME type will always be
text, so you can omit that. Because specifying a `mime` type without a category
(e.g., `lua` instead of `text/x-lua`) is invalid and potentially confusing, `BGML` uses the
`lang` attribute on `script` tags to indicate language instead.

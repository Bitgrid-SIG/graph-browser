
# Data-Driven Dynamic BGML Tags

url: `https://blah.com/blogs/d72502f4-bd5d-4213-97cf-f0edaabc3e7a/posts`
```html
<dynamic id="bloglistings" data-page="0" method="GET" href="/blogs/author"
        query-user="location.path.segment.2" query-page="this.page"
        query-pagination="20"
        >
    <container class="bloglist blogheader">
        <!-- ... -->
        <slot id=".0">
            <label level="3"><slot id="title" /></label>
            <label level="2"><slot id="author" /></label>
            <label level="5"><slot id="posted" /></label>
        </slot>
        <!-- ... -->
    </container>
    <container class="bloglist blogheader">
        <!-- ... -->
        <slot id=".1">
            <label level="3"><slot id="title" /></label>
            <label level="2"><slot id="author" /></label>
            <label level="5"><slot id="posted" /></label>
        </slot>
        <!-- ... -->
    </container>
    <container class="bloglist blogheader">
        <!-- ... -->
        <slot id=".*">
            <label level="3"><slot id="title" /></label>
            <label level="2"><slot id="author" /></label>
            <label level="5"><slot id="posted" /></label>
        </slot>
        <!-- ... -->
    </container>
    <container class="bloglist blogheader">
        <!-- ... -->
        <slot id=".20">
            <label level="3"><slot id="title" /></label>
            <label level="2"><slot id="author" /></label>
            <label level="5"><slot id="posted" /></label>
        </slot>
        <!-- ... -->
    </container>
    <button modify="#bloglistings" action="decrement" what="page" content="<" />
    <button modify="#bloglistings" action="increment" what="page" content=">" />
</dynamic>
```

url: `https://blah.com/blogs/d72502f4-bd5d-4213-97cf-f0edaabc3e7a/dc0059be-993e-4ab3-8e1e-40ac55f9054f`
```html
<dynamic id="blogpost" method="GET" href="/blogs/post" query-user="location.path.1"
        query-post="location.path.2"
        >
    <label level="1"><slot id="title" /></label>
    <label level="3"><slot id="author" /></label>
    <label level="3"><slot id="posted" /></label>
    <hr />
    <!-- blog post is likely to contain HTML. We don't want it treated as a string literal,
    so we mark it unsafe -->
    <slot unsafe="true" id="blogcontent" />
</dynamic>
```

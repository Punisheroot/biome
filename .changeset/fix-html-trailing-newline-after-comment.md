---
"@biomejs/biome": patch
---

Fixed the HTML formatter dropping the newline at the end of a document whose last thing is a comment:

```html
<div></div>
<!-- the last thing in the document -->
```

The file used to be written back without a final newline. A comment that trails the last element belongs to the end of the document, and the closing newline is now written after it rather than before.

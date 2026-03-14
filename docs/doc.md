# A Document where I store my understanding of different things.

### Segment

A segment is a part of the index that is stored on disk. It is immutable once written to object storage. A segment contains a header, a footer, and a body. The header contains metadata about the segment, such as the schema and field descriptors. The footer contains checksums and offsets for the data and index blobs in the body. The body contains the column data and index blobs.

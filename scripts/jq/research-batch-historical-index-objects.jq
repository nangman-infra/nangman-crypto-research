map({
  bucket:$bucket,
  key:.Key,
  uri:("s3://" + $bucket + "/" + .Key),
  last_modified:.LastModified,
  size:.Size
})

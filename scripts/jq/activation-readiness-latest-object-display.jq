if .key == null then
  {
    prefix:.prefix,
    lastModified:null,
    size:null,
    key:null
  }
else
  {
    prefix:.prefix,
    lastModified:.lastModified,
    size:.size,
    key:(.key | split("/") | .[0:4] | join("/") + "/...")
  }
end

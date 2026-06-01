(.Contents // [])
| sort_by(.LastModified, .Key)
| reverse
| .[0:$limit]

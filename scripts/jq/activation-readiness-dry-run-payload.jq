{
  Records: [{
    eventSource: "aws:s3",
    eventName: "ObjectCreated:Put",
    eventTime: "2026-05-23T00:00:00.000Z",
    s3: {
      bucket: { name: $bucket },
      object: {
        key: $key,
        eTag: "activation-readiness",
        sequencer: "0000000000000001"
      }
    }
  }]
}

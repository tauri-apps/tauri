(function (message) {
  return JSON.stringify(message, (_k, val) => {
    if (val instanceof Map) {
      let o = {};
      val.forEach((v, k) => o[k] = v);
      return o;
    } else {
      return val;
    }
  })
})

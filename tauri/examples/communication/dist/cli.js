document.getElementById('cli-matches').addEventListener('click', function () {
  window.__TAURI__.cli.getMatches()
    .then(registerResponse)
    .catch(registerResponse)
})

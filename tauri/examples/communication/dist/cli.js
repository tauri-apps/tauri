document.getElementById('cli-matches').addEventListener('click', function () {
  window.__TAURI__.cliMatches()
    .then(registerResponse)
    .catch(registerResponse)
})

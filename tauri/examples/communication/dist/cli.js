document.getElementById('cli-matches').addEventListener('click', function () {
  window.tauri.cliMatches()
    .then(registerResponse)
    .catch(registerResponse)
})

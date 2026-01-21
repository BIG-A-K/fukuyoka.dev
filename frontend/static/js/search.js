document.addEventListener('DOMContentLoaded', function() {
  const searchForm = document.getElementById('search-form');
  const searchInput = document.getElementById('search-input');
  const searchResults = document.getElementById('search-results');
  const searchStatus = document.getElementById('search-status');

  searchForm.addEventListener('submit', async function(e) {
    e.preventDefault();

    const query = searchInput.value.trim();
    if (!query) {
      showStatus('検索ワードを入力してください', 'error');
      return;
    }

    showStatus('検索中...', 'loading');
    searchResults.innerHTML = '';

    try {
      const response = await fetch('/api/search', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ text: query }),
      });

      const data = await response.json();
      console.log('API Response:', data);

      if (response.ok && data.results) {
        if (data.results.length === 0) {
          showStatus('該当する結果が見つかりませんでした', 'empty');
        } else {
          showStatus('', 'success');
          displayResults(data.results);
        }
      } else {
        showStatus(data.error || '検索に失敗しました', 'error');
      }
    } catch (error) {
      showStatus('エラー: ' + error.message, 'error');
    }
  });

  function displayResults(results) {
    console.log('Displaying results:', results);
    searchResults.innerHTML = results.map(result => {
      const url = result.url.toLowerCase() || '#';
      const thumbnail = result.thumbnail || '/favicon.png';
      return `
      <li>
        <a href="${url}" aria-label="${result.url.toLowerCase()}">
          <img src="${thumbnail}" alt="${result.title}" loading="lazy" />
          <div class="thumb-caption">
            <p class="thumb-title">${result.title}</p>
          </div>
        </a>
      </li>
    `;
    }).join('');
  }

  function showStatus(message, type) {
    searchStatus.textContent = message;
    searchStatus.className = 'search-status ' + type;
  }
});

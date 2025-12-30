document.addEventListener('DOMContentLoaded', function() {
  const fileInput = document.getElementById('file-input');
  const filePreview = document.getElementById('file-preview');
  const uploadBtn = document.getElementById('upload-btn');
  const uploadForm = document.getElementById('upload-form');
  const uploadStatus = document.getElementById('upload-status');
  const pushBtn = document.getElementById('push-btn');
  const syncStatus = document.getElementById('sync-status');
  const refreshBtn = document.getElementById('refresh-btn');
  const imagesList = document.getElementById('images-list');
  const diffBtn = document.getElementById('diff-btn');
  const diffStatus = document.getElementById('diff-status');
  const onlyLocalList = document.getElementById('only-local');
  const syncedList = document.getElementById('synced');

  // File selection preview
  fileInput.addEventListener('change', function() {
    filePreview.innerHTML = '';
    const files = this.files;

    if (files.length > 0) {
      uploadBtn.disabled = false;

      Array.from(files).forEach(file => {
        const reader = new FileReader();
        reader.onload = function(e) {
          const previewItem = document.createElement('div');
          previewItem.className = 'preview-item';
          previewItem.innerHTML = `
            <img src="${e.target.result}" alt="${file.name}">
            <span class="filename">${file.name}</span>
          `;
          filePreview.appendChild(previewItem);
        };
        reader.readAsDataURL(file);
      });
    } else {
      uploadBtn.disabled = true;
    }
  });

  // Upload form submission
  uploadForm.addEventListener('submit', async function(e) {
    e.preventDefault();

    const files = fileInput.files;
    if (files.length === 0) return;

    uploadBtn.disabled = true;
    showStatus(uploadStatus, 'loading', 'アップロード中...');

    const formData = new FormData();
    Array.from(files).forEach(file => {
      formData.append('files', file);
    });

    try {
      const response = await fetch('/api/akasha/upload', {
        method: 'POST',
        body: formData,
        credentials: 'include'
      });

      const result = await response.json();

      if (response.ok) {
        showStatus(uploadStatus, 'success', `${result.uploaded || files.length}件の画像をアップロードしました`);
        fileInput.value = '';
        filePreview.innerHTML = '';
        loadImages();
      } else {
        showStatus(uploadStatus, 'error', result.error || 'アップロードに失敗しました');
      }
    } catch (error) {
      showStatus(uploadStatus, 'error', 'エラー: ' + error.message);
    } finally {
      uploadBtn.disabled = false;
    }
  });

  // R2 push (upload to R2)
  pushBtn.addEventListener('click', async function() {
    pushBtn.disabled = true;
    showStatus(syncStatus, 'loading', 'R2にアップロード中...');

    try {
      const response = await fetch('/api/akasha/push', {
        method: 'POST',
        credentials: 'include'
      });

      const result = await response.json();

      if (result.status === 'ok') {
        showStatus(syncStatus, 'success', result.message || 'R2へのアップロードが完了しました');
      } else {
        showStatus(syncStatus, 'error', result.error || 'アップロードに失敗しました');
      }
    } catch (error) {
      showStatus(syncStatus, 'error', 'エラー: ' + error.message);
    } finally {
      pushBtn.disabled = false;
    }
  });

  // Diff check
  diffBtn.addEventListener('click', async function() {
    diffBtn.disabled = true;
    showStatus(diffStatus, 'loading', '差分を確認中...');

    try {
      const response = await fetch('/api/akasha/diff', { credentials: 'include' });
      const result = await response.json();

      if (result.status === 'ok') {
        showStatus(diffStatus, 'success', '差分を取得しました');
        displayDiffList(onlyLocalList, result.only_local, 'local');
        displayDiffList(syncedList, result.synced, 'synced');
      } else {
        showStatus(diffStatus, 'error', result.error || '差分取得に失敗しました');
      }
    } catch (error) {
      showStatus(diffStatus, 'error', 'エラー: ' + error.message);
    } finally {
      diffBtn.disabled = false;
    }
  });

  // Refresh images list
  refreshBtn.addEventListener('click', loadImages);

  // Load images on page load
  loadImages();

  async function loadImages() {
    try {
      const response = await fetch('/api/akasha/images', { credentials: 'include' });
      const result = await response.json();

      if (response.ok && result.images) {
        displayImages(result.images);
      } else {
        imagesList.innerHTML = '<p>画像を読み込めませんでした</p>';
      }
    } catch (error) {
      imagesList.innerHTML = '<p>エラー: ' + error.message + '</p>';
    }
  }

  function displayImages(images) {
    if (images.length === 0) {
      imagesList.innerHTML = '<p>アップロード済みの画像はありません</p>';
      return;
    }

    imagesList.innerHTML = images.map(img => `
      <div class="image-item">
        <img src="/api/akasha/local-image/${encodeURIComponent(img)}" alt="${img}" loading="lazy">
        <span class="image-name">${img}</span>
      </div>
    `).join('');
  }

  function showStatus(element, type, message) {
    element.className = 'status ' + type;
    element.textContent = message;
  }

  function displayDiffList(element, files, type) {
    if (!files || files.length === 0) {
      element.innerHTML = '<p class="empty">なし</p>';
      return;
    }

    const imgSrc = (filename) => {
      if (type === 'r2') {
        return `/photo/${encodeURIComponent(filename)}`;
      }
      return `/api/akasha/local-image/${encodeURIComponent(filename)}`;
    };

    element.innerHTML = files.map(filename => `
      <div class="diff-item ${type}">
        <img src="${imgSrc(filename)}" alt="${filename}" loading="lazy">
        <span>${filename}</span>
      </div>
    `).join('');
  }
});

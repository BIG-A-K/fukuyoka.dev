document.addEventListener('DOMContentLoaded', function() {
  const fileInput = document.getElementById('file-input');
  const filePreview = document.getElementById('file-preview');
  const uploadBtn = document.getElementById('upload-btn');
  const uploadForm = document.getElementById('upload-form');
  const uploadStatus = document.getElementById('upload-status');
  const syncBtn = document.getElementById('sync-btn');
  const syncStatus = document.getElementById('sync-status');
  const refreshBtn = document.getElementById('refresh-btn');
  const imagesList = document.getElementById('images-list');

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
      const response = await fetch('/api/upload', {
        method: 'POST',
        body: formData
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

  // R2 sync
  syncBtn.addEventListener('click', async function() {
    syncBtn.disabled = true;
    showStatus(syncStatus, 'loading', 'R2に同期中...');

    try {
      const response = await fetch('/api/sync', {
        method: 'POST'
      });

      const result = await response.json();

      if (response.ok) {
        showStatus(syncStatus, 'success', result.message || 'R2への同期が完了しました');
      } else {
        showStatus(syncStatus, 'error', result.error || '同期に失敗しました');
      }
    } catch (error) {
      showStatus(syncStatus, 'error', 'エラー: ' + error.message);
    } finally {
      syncBtn.disabled = false;
    }
  });

  // Refresh images list
  refreshBtn.addEventListener('click', loadImages);

  // Load images on page load
  loadImages();

  async function loadImages() {
    try {
      const response = await fetch('/api/images');
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
        <img src="/photo/${img}" alt="${img}" loading="lazy">
        <span class="image-name">${img}</span>
      </div>
    `).join('');
  }

  function showStatus(element, type, message) {
    element.className = 'status ' + type;
    element.textContent = message;
  }
});

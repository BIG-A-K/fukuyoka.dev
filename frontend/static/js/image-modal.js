// Simple image modal for images inside .post-body
(function () {
  function createModal() {
    const overlay = document.createElement('div');
    overlay.className = 'img-modal';
    overlay.innerHTML = `
      <div class="img-modal__backdrop" data-close></div>
      <figure class="img-modal__figure" role="dialog" aria-modal="true">
        <button class="img-modal__close" aria-label="Close" data-close>&times;</button>
        <img class="img-modal__img" alt="" />
        <figcaption class="img-modal__caption" hidden></figcaption>
      </figure>
    `;
    document.body.appendChild(overlay);
    return overlay;
  }

  function openModal(src, captionText) {
    const modal = document.querySelector('.img-modal') || createModal();
    const img = modal.querySelector('.img-modal__img');
    const caption = modal.querySelector('.img-modal__caption');
    img.src = src;
    img.alt = captionText || '';
    if (captionText) {
      caption.textContent = captionText;
      caption.hidden = false;
    } else {
      caption.hidden = true;
      caption.textContent = '';
    }
    modal.classList.add('open');
    document.documentElement.style.overflow = 'hidden';
    const onKey = (e) => {
      if (e.key === 'Escape') closeModal();
    };
    modal._onKey = onKey;
    document.addEventListener('keydown', onKey);
  }

  function closeModal() {
    const modal = document.querySelector('.img-modal');
    if (!modal) return;
    modal.classList.remove('open');
    document.documentElement.style.overflow = '';
    if (modal._onKey) {
      document.removeEventListener('keydown', modal._onKey);
      modal._onKey = null;
    }
  }

  function init() {
    const imgs = document.querySelectorAll('.post-body img');
    imgs.forEach((img) => {
      img.classList.add('zoomable');
      img.addEventListener('click', (ev) => {
        // Avoid navigating if image is inside a link
        if (img.closest('a')) ev.preventDefault();
        const src = img.dataset.fullsrc || img.currentSrc || img.src;
        const fig = img.closest('figure');
        const captionText = fig && fig.querySelector('figcaption') ? fig.querySelector('figcaption').textContent : img.alt;
        openModal(src, captionText);
      });
    });
    document.addEventListener('click', (e) => {
      if (e.target && e.target.hasAttribute('data-close')) {
        closeModal();
      }
    });
  }

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', init);
  } else {
    init();
  }
})();

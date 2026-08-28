/**
 * Jagad Shalawat Blog — Comment Handler
 * Minimal vanilla JS for comment form interactions (< 3KB)
 */
(function() {
    'use strict';

    // ── Reply to comment ────────────────────────────────────────
    document.addEventListener('click', function(e) {
        var btn = e.target.closest('[data-reply-to]');
        if (!btn) return;

        e.preventDefault();
        var parentId = btn.getAttribute('data-reply-to');
        var authorName = btn.getAttribute('data-reply-author');
        var form = document.getElementById('comment-form');
        var parentInput = document.getElementById('parent_id');
        var replyIndicator = document.getElementById('reply-indicator');
        var contentField = document.getElementById('comment-content');

        if (parentInput) parentInput.value = parentId;
        if (replyIndicator) {
            replyIndicator.textContent = 'Membalas ' + authorName;
            replyIndicator.style.display = 'block';
        }
        if (form) form.scrollIntoView({ behavior: 'smooth', block: 'center' });
        if (contentField) contentField.focus();
    });

    // ── Cancel reply ────────────────────────────────────────────
    document.addEventListener('click', function(e) {
        if (!e.target.closest('#cancel-reply')) return;

        e.preventDefault();
        var parentInput = document.getElementById('parent_id');
        var replyIndicator = document.getElementById('reply-indicator');

        if (parentInput) parentInput.value = '';
        if (replyIndicator) replyIndicator.style.display = 'none';
    });

    // ── Form validation ─────────────────────────────────────────
    var commentForm = document.getElementById('comment-form');
    if (commentForm) {
        commentForm.addEventListener('submit', function(e) {
            var content = document.getElementById('comment-content');
            if (content && content.value.trim().length < 3) {
                e.preventDefault();
                alert('Komentar terlalu pendek. Minimal 3 karakter.');
                return false;
            }
            if (content && content.value.length > 5000) {
                e.preventDefault();
                alert('Komentar terlalu panjang. Maksimal 5000 karakter.');
                return false;
            }

            // Disable submit button to prevent double-submit
            var submitBtn = commentForm.querySelector('button[type="submit"]');
            if (submitBtn) {
                submitBtn.disabled = true;
                submitBtn.textContent = 'Mengirim...';
            }
        });
    }
})();

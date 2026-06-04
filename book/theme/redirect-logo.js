// Redirect the mdBook title/logo link to the website landing page
(function () {
  var logo = document.querySelector('.sidebar .sidebar-header a');
  if (logo) {
    logo.href = '/rust-igraph/';
    logo.title = 'rust-igraph Home';
  }
})();

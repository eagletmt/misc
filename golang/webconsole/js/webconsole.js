jQuery(function($) {
  function fetch(id) {
    $.ajax({
      type: 'GET',
      url: '/executions/' + id + '.json',
      datatype: 'json',
      success: function(execution) {
        $('#command').text(execution.command);
        if (execution.status === -1) {
          startEventSource(execution);
        } else {
          renderExecution(execution);
        }
      }
    })
  }

  function renderExecution(execution) {
    $('#status').text('Exited with ' + execution.status);
    $('#output').text(execution.output);
  }

  function startEventSource(execution) {
    $('#status').text('Running')
    var output = $('#output');

    var eventSource = new EventSource('/executions/' + execution.id + '/console');
    eventSource.addEventListener('console-output', function(evt) {
      var data = JSON.parse(evt.data);
      output.text(output.text() + data.output);
    });
    eventSource.addEventListener('console-exit', function(evt) {
      eventSource.close();
      fetch(execution.id);
    });
  }

  fetch(location.pathname.match(/^\/executions\/(\d+)/)[1]);
});

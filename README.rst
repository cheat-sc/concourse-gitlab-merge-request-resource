=======================================
Concourse GitLab Merge Request Resource
=======================================

A concourse resource for checking merge requests.

Example
=======

This just gets merge requests and sets them `success`.

.. code-block:: yaml

   resource_types:
   - name: gitlab-mr
     type: registry-image
     check_every: 24h
     source:
      repository: registry.gitlab.com/cheatsc/concourse-gitlab-merge-request-resource/concourse-gitlab-merge-request-resource
      tag: latest

   resources:
   - name: merge-request
     type: gitlab-mr
     icon: gitlab
     source:
      uri: https://gitlab.com/cheatsc/concourse-gitlab-merge-request-resource.git
      private_token: 'sample'

   jobs:
   - name: build
     plan:
     - get: merge-request
       trigger: true
       version: every
     - put: merge-request
       params:
         resource_name: merge-request
         status: pending
     - put: merge-request
       params:
         resource_name: merge-request
         status: running
     - put: merge-request
       params:
         resource_name: merge-request
         status: success

Behavior
========

check
-----

Check that new merge request created or the HEAD of merge requests ware updated.

.. list-table:: Parameters
   :header-rows: 1

   * - Parameter
     - Type
     - Value
     - Description
   * - uri
     - String
     - Required
     - Repository url
   * - private_token
     - String
     - Required
     - Private token
   * - labels
     - List of String
     - Optional
     - Only check merge requests which has the given labels. If no labels specified (by default), check all merge requests.
   * - paths
     - List of String
     - Optional
     - Only check merge requests which includes the given path in the changes. If no paths specified (by default), check all merge requests.
   * - skip_draft
     - Boolean
     - Optional
     - Do not check draft merge requests.

in
--

Clone the repository at the commit id of a merge request

.. list-table:: Parameters
   :header-rows: 1

   * - Parameter
     - Type
     - Value
     - Description
   * - skip_clone
     - Boolean
     - Optional
     - Do not clone repository. This is used for the case which you only want to update the status of a merge request.

out
---

Update the status of a merge request.

.. list-table:: Parameters
   :header-rows: 1

   * - Parameter
     - Type
     - Value
     - Description
   * - resource_name
     - String
     - Required
     - name of resource.
   * - status
     - String
     - Required
     - status of merge request.
   * - pipeline_name
     - String
     - Optional
     - Set pipeline name. You can use the following parameters:

       * ``%BUILD_PIPELINE_NAME%``
       * ``%BUILD_JOB_NAME%``
       * ``%BUILD_TEAM_NAME%``
       * ``%BUILD_PIPELINE_INSTANCE_VARS%``

       By default, ``%BUILD_TEAM_NAME%::%BUILD_PIPELINE_NAME%`` is set.
   * - coverage
     - Float
     - Optional
     - Set coverage.

Build
=====

Build binaries
--------------

.. code-block:: fish

   $ cargo build --release


Build resource image with buildkit
----------------------------------

.. code-block:: fish

   $ buildctl build --frontend dockerfile.v0 --local dockerfile=. --local context=. --export-cache type=local,dest=$HOME/buildkit-cache --import-cache type=local,src=$HOME/buildkit-cache

License
=======

Licensed under either of

* `Apache License, Version 2.0 </LICENSE-APACHE-2.0>`_
* `MIT license </LICENSE-MIT>`_

at your option.

Contribution
============

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

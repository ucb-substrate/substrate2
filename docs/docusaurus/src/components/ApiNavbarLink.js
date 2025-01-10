import React from 'react';
import {useDocsVersionCandidates} from '@docusaurus/theme-common/internal';
import DefaultNavbarItem from '@theme/NavbarItem/DefaultNavbarItem';
import {getApiDocsUrl} from '@site/src/utils/versions';

const getVersionMainDoc = (version) =>
  version.docs.find((doc) => doc.id === version.mainDocId);
export default function ApiNavbarLink ({
  docsPluginId,
  ...props
}) {
  const version = useDocsVersionCandidates(docsPluginId)[0];
  return <DefaultNavbarItem {...props} label="API" to={getApiDocsUrl(version.label)} />;
}

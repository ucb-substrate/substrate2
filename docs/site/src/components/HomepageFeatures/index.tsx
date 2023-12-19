import clsx from 'clsx';
import Heading from '@theme/Heading';
import styles from './styles.module.css';

type FeatureItem = {
  title: string;
  Svg: React.ComponentType<React.ComponentProps<'svg'>>;
  description: JSX.Element;
};

const FeatureList: FeatureItem[] = [
  {
    title: 'Bring your Own Tools',
    Svg: require('@site/static/img/CrossedTools.svg').default,
    description: (
      <>
        Design, verify, and tapeout your chip with a tool suite of your choice 
        using Substrate's extensible plugin system.
      </>
    ),
  },
  {
    title: '100% Performant Rust Code',
    Svg: require('@site/static/img/rust_logo.svg').default,
    description: (
      <>
        Generators can be written entirely in Rust, providing high performance alongside memory safety and type checking.
      </>
    ),
  },
  {
    title: 'Open Source',
    Svg: require('@site/static/img/Globe_icon.svg').default,
    description: (
      <>
        The core of Substrate is open source, meaning anyone can write a circuit generator without an expensive license.
      </>
    ),
  },
];

function Feature({title, Svg, description}: FeatureItem) {
  return (
    <div className={clsx('col col--4')}>
      <div className="text--center">
        <Svg className={clsx("featureSvg", styles.featureSvg)} role="img" />
      </div>
      <div className="text--center padding-horiz--md">
        <Heading as="h3">{title}</Heading>
        <p>{description}</p>
      </div>
    </div>
  );
}

export default function HomepageFeatures(): JSX.Element {
  return (
    <section className={styles.features}>
      <div className="container">
        <div className="row">
          {FeatureList.map((props, idx) => (
            <Feature key={idx} {...props} />
          ))}
        </div>
      </div>
    </section>
  );
}

type SetupBlockProps = {
	title: string;
	children: React.ReactNode;
};

export function SetupBlock({ title, children }: SetupBlockProps) {
	return (
		<section>
			<h2 className="mb-2 text-sm font-semibold">{title}</h2>
			{children}
		</section>
	);
}

function NavBar() {
  return (
    <nav>
      {/* Progress bar for a tick */}
      <div className="bg-slate-400/20 h-2">
        <div className="h-full w-[30%] bg-red-400/80" />
      </div>
      <div className="p-8 flex flex-row items-center gap-6">
        <div className="text-xl font-bold">Kriger</div>
        <a>Flags</a>
        <a>Executions</a>
        <a>Config</a>
        <div className="flex-1" />
        {/* Current tick + remaining tick time */}
        <div className="font-bold">
          Tick 2 <span className="font-normal text-slate-300">(20s)</span>
        </div>
      </div>
    </nav>
  );
}
export default NavBar;
